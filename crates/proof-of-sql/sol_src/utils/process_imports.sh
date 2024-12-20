#!/bin/bash
set -e  # Exit on any error

# Verify correct usage with directory argument
if [ $# -ne 1 ] || [ ! -d "$1" ]; then
    echo "Usage: $0 <directory>" >&2
    exit 1
fi

# Track overall processing status
status=0

# Process all .pre.sol files in directory and subdirectories
while IFS= read -r -d '' file; do
    outfile="${file%.pre.sol}.post.sol"
    echo "Processing $file -> $outfile"
    
    # Process file with AWK script
    # - Handles IMPORT-YUL directives
    # - Resolves paths relative to source file
    # - Validates function definitions
    # - Maintains original formatting
    if ! awk '
    # Track imported functions and handle errors
    /^[[:space:]]*\/\/[[:space:]]*IMPORT-YUL/ {
        if (NF != 3) error_exit("Invalid import directive format")
        # Resolve import path relative to source file
        import_file = FILENAME
        sub(/\/[^\/]*$/, "", import_file)
        import_file = import_file "/" $3
        if (system("test -r " import_file) != 0) 
            error_exit("File not found: " import_file)
        import_line = $0
        next
    }

    # Process function definitions after import directive
    import_file && /^[[:space:]]*function/ {
        split($2, a, "(")
        sub("IMPORT-YUL", "IMPORTED-YUL", import_line)
        print import_line "::" a[1]
        
        # Extract function from import file
        braces = found = 0
        while ((getline line < import_file) > 0) {
            if (!found && line ~ ("[[:space:]]*function[[:space:]]+" a[1] "\\("))
                found = 1
            if (found) {
                print line
                gsub(/[^{}]/, "", line)
                braces += (gsub(/{/,"{",line)) - (gsub(/}/,"}",line))
                if (braces < 0) error_exit("Unmatched closing brace in " a[1])
                if (!braces) break
            }
        }
        
        # Validate function extraction
        if (!found) error_exit("Function " a[1] " not found in " import_file)
        if (braces) error_exit("Unclosed braces in " a[1])
        close(import_file)
        import_file = ""
        next
    }

    # Pass through all other lines unchanged
    !import_file { print }

    function error_exit(m) { printf("Error: %s\n", m) > "/dev/stderr"; exit 1 }
    ' "$file" > "$outfile.tmp"; then
        echo "Error processing $file" >&2
        rm -f "$outfile.tmp"
        status=1
        continue
    fi
    
    # Atomically replace output file
    mv "$outfile.tmp" "$outfile"
done < <(find "$1" -type f -name "*.pre.sol" -print0)

exit $status
