# Yul and Solidity Import Preprocessor

This script processes Solidity files containing special Yul import directives and replaces function declarations with the actual imported Yul code. It also updates Solidity import paths from `.pre.sol` to `.post.sol`.

## Usage

```bash
./preprocess_yul_imports.sh <directory>
```

The script will recursively process all `*.pre.sol` files in the specified directory and create corresponding `*.post.sol` files with the processed imports.

## How it Works

1. The script finds all `*.pre.sol` files in the given directory.
2. For each file, it looks for special import directives in the format:
   ```solidity
   // IMPORT-YUL path/to/file.sol
   function someFunction() { ... }
   ```
3. It also updates Solidity import paths from `.pre.sol` to `.post.sol`:
   ```solidity
   import {SomeLibrary} from "./SomeLibrary.post.sol";
   ```

4. When it finds an import directive:
   - Resolves the import path relative to the source file.
   - Locates the specified function in the imported file.
   - Replaces the original function with the imported implementation.
   - Adds a comment indicating the source: `// IMPORTED-YUL path/to/file.sol::functionName`

## Example

Source file (`yul_utils.sol`):
```solidity
contract YulUtils {
    function doSomething() public pure returns (uint256) {
        assembly {
            // Pure Yul utility functions
            function keccak256_uint(v) -> h {
                mstore(0x00, v)
                h := keccak256(0x00, 0x20)
            }
            
            function encode_packed_uint(v) -> ptr, len {
                let start := mload(0x40)
                ptr := start
                switch gt(v, 0)
                case 0 { len := 1 }
                default {
                    len := 0
                    let tmp := v
                    for {} gt(tmp, 0) { tmp := div(tmp, 256) } {
                        len := add(len, 1)
                    }
                }
                mstore(ptr, shl(mul(sub(32, len), 8), v))
            }
        }
    }
}
```

Main contract (`Contract.pre.sol`):
```solidity
import {YulUtils} from "../libs/YulUtils.pre.sol";

contract MyContract {
    function processData(uint256 value) public pure returns (bytes32) {
        assembly {
            // IMPORT-YUL ../libs/yul_utils.sol
            function keccak256_uint(v) -> h {}
            
            // IMPORT-YUL ../libs/yul_utils.sol  
            function encode_packed_uint(v) -> ptr, len {}
            
            let encoded_ptr, encoded_len := encode_packed_uint(value)
            let hash := keccak256_uint(value)
            mstore(0x00, hash)
            return(0x00, 0x20)
        }
    }
}
```

After processing (`Contract.post.sol`):
```solidity
import {YulUtils} from "../libs/YulUtils.post.sol";

contract MyContract {
    function processData(uint256 value) public pure returns (bytes32) {
        assembly {
            // IMPORTED-YUL ../libs/yul_utils.sol::keccak256_uint
            function keccak256_uint(v) -> h {
                mstore(0x00, v)
                h := keccak256(0x00, 0x20)
            }
            
            // IMPORTED-YUL ../libs/yul_utils.sol::encode_packed_uint
            function encode_packed_uint(v) -> ptr, len {
                let start := mload(0x40)
                ptr := start
                switch gt(v, 0)
                case 0 { len := 1 }
                default {
                    len := 0
                    let tmp := v
                    for {} gt(tmp, 0) { tmp := div(tmp, 256) } {
                        len := add(len, 1)
                    }
                }
                mstore(ptr, shl(mul(sub(32, len), 8), v))
            }
            
            let encoded_ptr, encoded_len := encode_packed_uint(value)
            let hash := keccak256_uint(value)
            mstore(0x00, hash)
            return(0x00, 0x20)
        }
    }
}
```

## Error Handling

The script will fail with an error if:
- The specified directory doesn't exist.
- An imported file cannot be read.
- A referenced function cannot be found in the imported file.

Error messages are written to stderr and the script exits with a non-zero status code on any error.
