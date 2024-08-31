import { verify } from "../../../verifier-wasm-artifacts/verifier_wasm.js";

const query = await Deno.readTextFile("param_query.txt");
const schema = await Deno.readTextFile("param_schema.txt");
const query_commitments = await Deno.readFile("param_query_commitments.bin");
const proof = await Deno.readFile("param_proof.bin");
const serialized_result = await Deno.readFile("param_serialized_result.bin");
const verifier_setup = await Deno.readFile("param_verifier_setup.bin");
const sigma = await Deno.readTextFile("param_sigma.txt");

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
