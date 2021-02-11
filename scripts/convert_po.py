#!/usr/bin/env python3

import os
import sys
import io
import polib
import json

if len(sys.argv) < 3:
    print("Usage: convert_po input.po output.json")
    sys.exit(2)

src_file = sys.argv[1]
dest_file = sys.argv[2]

if not os.path.isfile(src_file):
    print("File doesn't exist: " + src_file)
    sys.exit(2)

po = polib.pofile(src_file)

def obj_to_dict(obj):
    return obj.__dict__

with io.open(dest_file, "w+", encoding = "utf-8") as json_file:
    json.dump(po,
              json_file,
              indent = 4,
              ensure_ascii = False,
              default = obj_to_dict)
