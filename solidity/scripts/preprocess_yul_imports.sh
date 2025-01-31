#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ] || [ ! -d "$1" ]; then
    echo "Usage: $0 <directory>" >&2
    exit 1
fi

status=0

while IFS= read -r -d '' file; do
    outfile="${file%.pre.sol}.post.sol"
    echo "Processing $file -> $outfile"
    
    if ! gawk '
    # Handle Yul imports
    /\/\/ IMPORT-YUL/ {
        import_path = $3
        # Resolve actual file path for reading
        src_dir = FILENAME
        sub(/\/[^\/]*$/, "", src_dir)
        import_file = src_dir "/" import_path
        
        if (system("test -r " import_file)) {
            print "Error: File " import_file " not found" > "/dev/stderr"
            exit 1
        }
        
        # Read the function signature which may span multiple lines
        signature = ""
        do {
            getline
            signature = signature " " $0
        } while (signature !~ /{/)
        
        # Extract function name from the complete signature
        match(signature, /function[[:space:]]+([[:alnum:]_]+)/, m)
        print "            // IMPORTED-YUL " import_path "::" m[1]
        print "            function exclude_coverage_start_" m[1] "() {} // solhint-disable-line no-empty-blocks"
        
        # Skip original function if not empty
        if (!(signature ~ /{[[:space:]]*}/)) {
            depth = 1
            while (depth > 0) {
                getline
                depth += gsub(/{/, "{") - gsub(/}/, "}")
            }
        }
        
        # Find and print the imported function
        found = 0
        joined_lines = ""
        while ((getline line < import_file) > 0) {
            # Fixed regex pattern concatenation
            pattern = "^[[:space:]]*function[[:space:]]+" m[1] "[[:space:]]*(\\(|$)"
            if (line ~ pattern) {
                found = 1
                # Handle multiline function signature
                joined_lines = line
                while (joined_lines !~ /{/) {
                    getline line < import_file
                    joined_lines = joined_lines "\n" line
                }
                print joined_lines
                depth = 1
                while (depth > 0 && (getline line < import_file) > 0) {
                    depth += gsub(/{/, "{", line) - gsub(/}/, "}", line)
                    print line
                }
                break
            }
        }

        print "            function exclude_coverage_stop_" m[1] "() {} // solhint-disable-line no-empty-blocks"
        close(import_file)
        
        if (!found) {
            print "Error: Function " m[1] " not found in " import_file > "/dev/stderr"
            exit 1
        }
        next
    }
    # Handle Solidity imports
    /^import/ {
        import_line = $0
        while (import_line !~ /;[[:space:]]*$/) {
            getline
            import_line = import_line "\n" $0
        }
        gsub(/\.pre\.sol/, ".post.sol", import_line)
        print import_line
        next
    }
    { print }
    ' "$file" > "$outfile.tmp"; then
        echo "Error processing $file" >&2
        rm -f "$outfile.tmp"
        status=1
        continue
    fi
    
    mv "$outfile.tmp" "$outfile"
done < <(find "$1" -type f -name "*.pre.sol" -print0)

exit $status
