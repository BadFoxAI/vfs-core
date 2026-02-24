import sys
import os

if len(sys.argv) < 2:
    print("Usage: python3 patch.py <target_file>")
    sys.exit(1)

target_file = sys.argv[1]

if not os.path.exists("patch_match.txt") or not os.path.exists("patch_replace.txt"):
    print("Error: patch_match.txt and patch_replace.txt must exist.")
    sys.exit(1)

with open("patch_match.txt", "r") as f:
    match_text = f.read().strip()

with open("patch_replace.txt", "r") as f:
    replace_text = f.read().strip()

with open(target_file, "r") as f:
    content = f.read()

if match_text not in content:
    print(f"Error: Match text not found in {target_file}")
    # Debug aid: print first 50 chars of match
    print(f"Searching for: {match_text[:50]}...")
    sys.exit(1)

new_content = content.replace(match_text, replace_text)

with open(target_file, "w") as f:
    f.write(new_content)

print(f"Successfully patched {target_file}")
