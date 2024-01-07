#!/bin/bash

find tests -type f -name "*out.rb" | xargs -L 1 ruby -c
