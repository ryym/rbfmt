# frozen_string_literal: true

class Main
  def run
    prism_source_path = './target/debug/build/ruby-prism-e22c044bec47bbd1/out/bindings.rs'
    code_gen = MeaningCodeGenerator.new
    code_gen.run(prism_source_path)
  end
end

class MeaningCodeGenerator
  def run(src_file_path)
    lines = File.read(src_file_path).chomp.lines.map(&:chomp)

    impls = extract_node_impls(lines)
    functions_list = extract_target_functions(impls)

    pattern_match_branches = impls.map.with_index do |impl, i|
      functions = functions_list[i]
      build_pattern_match_branch(impl, functions)
    end

    autogen_source = <<~RUST
      // NOTE: This is auto-generated by scripts/meaning_autogen.rb
      impl super::Meaning {
          pub(super) fn node(&mut self, node: &prism::Node) {
              match node {
                  #{pattern_match_branches.join("\n")}
              }
          }
      }
    RUST

    autogen_path = 'src/meaning/autogen.rs'
    File.write(autogen_path, autogen_source)
    system("rustfmt #{autogen_path}", exception: true)
  end

  private

  def extract_node_impls(lines)
    regex_impl_start = /^impl<'pr> ([^<]+Node)<'pr> \{$/
    regex_impl_end = /^\}$/
    regex_fn_def = /^\s+pub fn ([^(]+)\([^)]+\) -> ([^{]+) \{$/

    impls = []
    state = { type: :none }
    found = 0

    lines.each do |line|
      case state[:type]
      when :none
        m = line.match(regex_impl_start)
        if m != nil
          state = {
            type: :in_impl,
            impl: { name: m.captures[0], fns: [] },
          }
        end
      when :in_impl
        if line == '}'
          impls << state[:impl]
          state = { type: :none }
        else
          m = line.match(regex_fn_def)
          if m != nil
            found += 1
            state[:impl][:fns] << {
              name: m.captures[0],
              return_type: m.captures[1],
            }
          end
        end
      end
    end

    impls
  end

  def extract_target_functions(impls)
    non_target_names = %w[as_node location flags depth maximum number opening_loc closing_loc]
    non_target_return_types = [
      "bool",
      "ConstantId<'pr>",
      "ConstantList<'pr>",
      "Option<ConstantId<'pr>>",
    ]
    should_ignore = ->(fn) {
      non_target_names.include?(fn[:name]) ||
        non_target_return_types.include?(fn[:return_type])
    }
    impls.map do |impl|
      impl[:fns].reject do |fn|
        should_ignore.call(fn)
      end
    end
  end

  def build_pattern_match_branch(impl, functions)
    fields = functions.filter_map { build_field_line(impl, _1) }
    if fields.empty?
      if impl[:name] == 'NumberedParametersNode'
        <<~RUST
          prism::Node::#{impl[:name]} { .. } => {
              self.numbered_parameters_node("#{impl[:name]}");
          }
        RUST
      else
        <<~RUST
          prism::Node::#{impl[:name]} { .. } => {
              self.atom_node("#{impl[:name]}", node);
          }
        RUST
      end
    else
      snaked_name = to_snake(impl[:name])
      <<~RUST
        prism::Node::#{impl[:name]} { .. } => {
            let node = node.as_#{snaked_name}().unwrap();
            self.start_node("#{impl[:name]}");
            #{fields.join("\n    ")}
            self.end_node();
        }
      RUST
    end
  end

  def build_field_line(impl, fn)
    name = fn[:name]

    case impl[:name]
    when 'ForNode'
      case name
      when 'do_keyword_loc'
        return nil
      end
    when 'MultiWriteNode'
      case name
      when 'lparen_loc', 'rparen_loc'
        return nil
      end
    when 'DefNode'
      case name
      when 'lparen_loc', 'rparen_loc'
        return nil
      end
    when 'IfNode', 'UnlessNode'
      case name
      when 'then_keyword_loc'
        return nil
      end
    end

    case fn[:return_type]
    when "Node<'pr>"
      %Q{self.node_field("#{name}", node.#{name}());}
    when "NodeList<'pr>"
      %Q{self.list_field("#{name}", node.#{name}());}
    when "Option<Node<'pr>>"
      %Q{self.opt_field("#{name}", node.#{name}());}
    when "Location<'pr>"
      case name
      when 'content_loc', 'value_loc'
        %Q{self.string_content(node.#{name}());}
      when 'message_loc'
        %Q{self.message_loc_field(Some(node.#{name}()));}
      else
        %Q{self.opt_loc_field("#{name}", Some(node.#{name}()));}
      end
    when "Option<Location<'pr>>"
      case name
      when 'call_operator_loc'
        %Q{self.call_operator_loc_field(node.#{name}());}
      when 'message_loc'
        %Q{self.message_loc_field(node.#{name}());}
      else
        %Q{self.opt_loc_field("#{name}", node.#{name}());}
      end
    when /^([a-zA-Z]+)Node<'pr>$/
      %Q{self.node_field("#{name}", node.#{name}().as_node());}
    when /^Option<([a-zA-Z]+)Node<'pr>>$/
      %Q{self.opt_field("#{name}", node.#{name}().map(|n| n.as_node()));}
    else
      raise "unexpected return_type: #{fn}"
    end
  end

  def to_snake(str)
    str
      .gsub(/([A-Z]+)([A-Z][a-z])/, '\1_\2')
      .gsub(/([a-z\d])([A-Z])/, '\1_\2')
      .downcase
  end
end

Main.new.run
