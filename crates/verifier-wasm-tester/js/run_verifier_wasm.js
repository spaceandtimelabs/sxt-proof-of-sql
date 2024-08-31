import { verify } from "../../../verifier-wasm-artifacts/verifier_wasm.js";

if (Deno.args.length != 1) {
    console.log("Path to a folder that contains files with verifier inputs must be specified.")
    Deno.exit(1);
}

const input_dir = Deno.args[0];
console.log("Reading verifier inputs from " + input_dir);

const query = await Deno.readTextFile(input_dir + "/param_query.txt");
const schema = await Deno.readTextFile(input_dir + "/param_schema.txt");
const query_commitments = await Deno.readFile(input_dir + "/param_query_commitments.bin");
const proof = await Deno.readFile(input_dir + "/param_proof.bin");
const serialized_result = await Deno.readFile(input_dir + "/param_serialized_result.bin");
const verifier_setup = await Deno.readFile(input_dir + "/param_verifier_setup.bin");
const sigma = await Deno.readTextFile(input_dir + "/param_sigma.txt");

let ret = verify(
    query,
    schema,
    query_commitments,
    proof,
    serialized_result,
    verifier_setup,
    sigma
);

switch (ret) {
    case 0:
        console.log("Verification SUCCESS");
        break;
    case 1:
        console.log("Verification FAIL");
        break;
    case 2:
        console.log("Parameter parsing failed");
        break;
    default:
        console.log("Unexpected return value: " + ret);
}
