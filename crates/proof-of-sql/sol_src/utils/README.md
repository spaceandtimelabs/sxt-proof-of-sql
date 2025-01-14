# YUL Pre-processor Import Tool

A shell script that processes Solidity/YUL files with import directives to include functions from external files.

## Purpose

The tool allows you to maintain YUL functions across multiple files and combine them during preprocessing. 
Source files use the `.pre.sol` extension and are processed into `.sol` files.

## Usage

```bash
./process_imports.sh <directory>
```

This will:
1. Find all `.pre.sol` files in the directory and subdirectories
2. Process each file and create corresponding `.sol` output
3. Report any errors during processing

## Import Syntax

To import functions from another file, use the following directive:

```yul
// IMPORT-YUL path/to/file.sol
function someFunction() {
    // This declaration will be replaced with the actual function
}
```

The import directive will be transformed into a comment showing what was imported:
```yul
// IMPORTED-YUL path/to/file.sol::someFunction
```

## Example

Directory structure:
```
src/
  ├── utils/
  │   └── math.sol
  ├── storage/
  │   └── array.sol
  └── main.pre.sol
```

utils/math.sol:
```yul
function add(x, y) -> sum {
    sum := add(x, y)
}

function multiply(x, y) -> product {
    product := mul(x, y)
}
```

storage/array.sol:
```yul
function array_push(array_ptr, value) {
    let length := mload(array_ptr)
    mstore(add(array_ptr, mul(add(length, 1), 32)), value)
    mstore(array_ptr, add(length, 1))
}
```

main.pre.sol:
```yul
// IMPORT-YUL utils/math.sol
function add(x, y) {}

// IMPORT-YUL utils/math.sol
function multiply(x, y) {}

// IMPORT-YUL storage/array.sol
function array_push(ptr, val) {}

let numbers_ptr := mload(0x40)
mstore(numbers_ptr, 0) // Initialize array length
let sum := add(1, 2)
array_push(numbers_ptr, sum)
array_push(numbers_ptr, multiply(sum, 3))
```

After running `./process_imports.sh src`:

Generated main.sol:
```yul
// IMPORTED-YUL utils/math.sol::add
function add(x, y) -> sum {
    sum := add(x, y)
}

// IMPORTED-YUL utils/math.sol::multiply
function multiply(x, y) -> product {
    product := mul(x, y)
}

// IMPORTED-YUL storage/array.sol::array_push
function array_push(array_ptr, value) {
    let length := mload(array_ptr)
    mstore(add(array_ptr, mul(add(length, 1), 32)), value)
    mstore(array_ptr, add(length, 1))
}

let numbers_ptr := mload(0x40)
mstore(numbers_ptr, 0) // Initialize array length
let sum := add(1, 2)
array_push(numbers_ptr, sum)
array_push(numbers_ptr, multiply(sum, 3))
```

Directory after processing:
```
src/
  ├── utils/
  │   └── math.sol
  ├── storage/
  │   └── array.sol
  ├── main.pre.sol
  └── main.sol    # Generated output
```

## Error Handling

The script will:
- Report any processing errors
- Clean up temporary files on failure
- Exit with non-zero status if any file fails
- Maintain original files on error
