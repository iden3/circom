#!/usr/bin/env python3
"""Run the self-contained `-cbc` test suite.

Each test is a single .circom file with no `include` (circomlib2 templates it
needs are inlined verbatim, see tools/inline_circomlib.py) and is compiled
WITHOUT `-l`. Expectations live in header comments:

    // @expect PASS | FAIL | COMPILE_ERROR
    // @certified <n>                     number of certified instances
    // @trusted <Instance()>              one line per distinct trusted instance (exact
                                          set; no line = nothing trusted)
    // @fail <Instance()> <CODE> ...      one line per non-certifiable instance with
                                          its error codes (exact set and multiset)
    // @error <CODE>                      for COMPILE_ERROR: circom's error code (e.g. T3001)
    // @args <extra circom flags>         optional
    // @known-bug <description>           the expectations above describe the CORRECT
                                          behaviour, which -cbc does not implement yet:
                                          a mismatch is reported as `xfail` (not a
                                          failure); a match as `XPASS` (remove the marker)

For PASS/FAIL the expectations are compared against the final
"Certification report" block. COMPILE_ERROR means circom rejects the file
before the certifier runs (no report is printed).

Usage:
    run_tests.py [--circom PATH] [-v] [filters...]
"""

import argparse
import collections
import pathlib
import re
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
DEFAULT_CIRCOM = HERE.parent / "target" / "debug" / "circom"
ANSI = re.compile(r"\x1b\[[0-9;]*m")
FAIL_LINE = re.compile(r"^(.*?)\s+((?:[A-Z]{1,3}\d+\s*)+)$")


def strip_comments(src):
    src = re.sub(r"/\*.*?\*/", " ", src, flags=re.S)
    return re.sub(r"//[^\n]*", " ", src)


def parse_expectations(path):
    src = path.read_text()
    exp = {"expect": None, "certified": None, "trusted": [], "fail": {}, "args": [], "known_bug": None, "error": None}
    for m in re.finditer(r"^//\s*@([\w-]+)\s*(.*?)\s*$", src, re.M):
        key, val = m.group(1).replace("-", "_"), m.group(2)
        if key == "expect":
            exp["expect"] = val
        elif key == "certified":
            exp["certified"] = int(val)
        elif key == "trusted":
            exp["trusted"].append(val)
        elif key == "fail":
            fm = FAIL_LINE.match(val)
            if not fm:
                raise ValueError(f"{path}: bad @fail line {val!r}")
            exp["fail"][fm.group(1)] = sorted(fm.group(2).split())
        elif key == "error":
            exp["error"] = val
        elif key == "args":
            exp["args"] = val.split()
        elif key == "known_bug" or key == "known":
            exp["known_bug"] = val
    if exp["expect"] not in ("PASS", "FAIL", "COMPILE_ERROR"):
        raise ValueError(f"{path}: missing or bad @expect")
    if re.search(r"^\s*include\b", strip_comments(src), re.M):
        raise ValueError(f"{path}: tests must be self-contained (found `include`)")
    return exp


def parse_report(out):
    """Parse the final Certification report block, or None if absent."""
    if "Certification report" not in out:
        return None
    lines = out[out.rindex("Certification report"):].splitlines()
    rep = {"verdict": None, "certified": None, "trusted": [], "fail": {}}
    section, current = None, None
    for line in lines[1:]:
        s = line.strip()
        if s.startswith("Certified:"):
            rep["certified"] = int(s.split(":")[1])
            section = "certified"
        elif s.startswith("Trusted (circomlib):"):
            section = "trusted"
        elif s.startswith("Not certifiable:"):
            section = "fail"
        elif s.startswith("Certifiable:"):
            rep["verdict"] = "PASS" if "PASS" in s else "FAIL"
            break
        elif s.startswith("- ") and section == "trusted":
            rep["trusted"].append(s[2:])
        elif s.startswith("- ") and section == "fail":
            current = re.sub(r"\s*\(\d+ error\(s\)\):$", "", s[2:])
            rep["fail"][current] = []
        elif s.startswith("[") and section == "fail" and current is not None:
            rep["fail"][current].append(s[1:s.index("]")])
    for k in rep["fail"]:
        rep["fail"][k].sort()
    return rep


def compare(exp, rep, out):
    problems = []
    if exp["expect"] == "COMPILE_ERROR":
        if rep is not None:
            problems.append(f"expected a compile error, got a certification report ({rep['verdict']})")
        elif exp["error"] and f"error[{exp['error']}]" not in out:
            problems.append(f"expected circom error {exp['error']}, not found in the output")
        return problems
    if rep is None:
        return ["no certification report (compile error?)"]
    if rep["verdict"] != exp["expect"]:
        problems.append(f"verdict {rep['verdict']}, expected {exp['expect']}")
    if exp["certified"] is not None and rep["certified"] != exp["certified"]:
        problems.append(f"certified {rep['certified']}, expected {exp['certified']}")
    if set(rep["trusted"]) != set(exp["trusted"]):
        problems.append(f"trusted {sorted(rep['trusted'])}, expected {sorted(exp['trusted'])}")
    if rep["fail"] != exp["fail"]:
        problems.append(f"not certifiable {rep['fail']}, expected {exp['fail']}")
    return problems


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--circom", default=str(DEFAULT_CIRCOM))
    ap.add_argument("-v", "--verbose", action="store_true", help="print circom output of failing tests")
    ap.add_argument("filters", nargs="*", help="only run tests whose path contains one of these")
    args = ap.parse_args()

    tests = sorted(p for p in HERE.rglob("*.circom") if "tools" not in p.relative_to(HERE).parts)
    if args.filters:
        tests = [t for t in tests if any(f in str(t.relative_to(HERE)) for f in args.filters)]

    passed, failed, xfail, xpass = 0, [], [], []
    with tempfile.TemporaryDirectory() as outdir:
        for test in tests:
            name = test.relative_to(HERE).as_posix()
            try:
                exp = parse_expectations(test)
            except ValueError as err:
                failed.append(name)
                print(f"ERROR {name}: {err}")
                continue
            cmd = [args.circom, str(test), "-cbc", "-o", outdir] + exp["args"]
            proc = subprocess.run(cmd, capture_output=True, text=True, cwd=HERE)
            out = ANSI.sub("", proc.stdout + proc.stderr)
            problems = compare(exp, parse_report(out), out)
            if exp["known_bug"] and problems:
                xfail.append(name)
                print(f"xfail {name}  (known bug: {exp['known_bug']})")
                if args.verbose:
                    for p in problems:
                        print(f"        {p}")
            elif exp["known_bug"]:
                xpass.append(name)
                print(f"XPASS {name}  (known bug seems fixed: remove @known-bug)")
            elif problems:
                failed.append(name)
                print(f"FAIL  {name}")
                for p in problems:
                    print(f"        {p}")
                if args.verbose:
                    print("\n".join("      | " + l for l in out.splitlines()))
            else:
                passed += 1
                print(f"ok    {name}")

    print(f"\n{passed} passed, {len(failed)} failed, {len(xfail)} known bugs (xfail), "
          f"{len(xpass)} XPASS, {len(tests)} total")
    sys.exit(1 if failed or xpass else 0)


if __name__ == "__main__":
    main()
