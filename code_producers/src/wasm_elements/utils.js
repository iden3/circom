const fnv = require("fnv-plus");

module.exports.fnvHash = fnvHash;
module.exports.toArray32 = toArray32;
module.exports.fromArray32 = fromArray32;
module.exports.flatArray = flatArray;

function toArray32(s,size) {
    const res = []; //new Uint32Array(size); //has no unshift
    let rem = BigInt(s);
    const radix = BigInt(0x100000000);
    while (rem) {
        res.unshift( Number(rem % radix));
        rem = rem / radix;
    }
    if (size) {
	var i = size - res.length;
	while (i>0) {
	    res.unshift(0);
	    i--;
	}
    }
    return res;
}

function fromArray32(arr) { //returns a BigInt
    var res = BigInt(0);
    const radix = BigInt(0x100000000);
    for (let i = 0; i<arr.length; i++) {
        res = res*radix + BigInt(arr[i]);
    }
    return res;
}

function flatArray(a) {
    var res = [];
    fillArray(res, a);
    return res;

    function fillArray(res, a) {
        if (Array.isArray(a)) {
            for (let i=0; i<a.length; i++) {
                fillArray(res, a[i]);
            }
        } else {
            res.push(a);
        }
    }
}

function fnvHash(str) {
    return fnv.hash(str, 64).hex();
}
