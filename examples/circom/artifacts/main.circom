pragma circom 2.0.3;

// include "https://github.com/0xPARC/circom-secp256k1/blob/master/circuits/bigint.circom";

template Example () {
    signal input a;
    signal input b;

    a === b;
}

component main { public [ a ] } = Example();

/* INPUT = {
    "a": "5",
    "b": "5"
} */