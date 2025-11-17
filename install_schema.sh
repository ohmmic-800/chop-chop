#!/usr/bin/env bash

schema_dir=$HOME/.local/share/glib-2.0/schemas
mkdir -p "$schema_dir"
cp gschema.xml "$schema_dir/com.ohmm-software.Chop-Chop.gschema.xml"
glib-compile-schemas "$schema_dir"
