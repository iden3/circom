#!/usr/bin/env python3
"""
Testing script for compilation coverage.

Receives a circom file to be sent to the compiler and an output path for the report.
"""
import glob
import json
import os
import os.path
import re
import subprocess
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Tuple, List, Match

import click


GLOB = "tests/vulnerabilities/**/test.json"


def setup_dirs(src: str, out: str) -> Tuple[str, str, TemporaryDirectory]:
    src = os.path.realpath(src)
    out = os.path.realpath(out)
    tmp = TemporaryDirectory()
    return src, out, tmp


def check_is_missing_feature(message: str) -> Match[str] | None:
    check = re.compile(r"thread 'main' panicked at 'not yet implemented', (.*):(.*):(.*)")
    return check.match(message)


def check_is_other_panic(message: str) -> Match[str] | None:
    check = re.compile(r"thread 'main' panicked at (.*), (.*):(.*):(.*)")
    return check.match(message)


def is_circom_error(message: str) -> bool:
    return re.compile(r"^error\[.*\]:").match(message) is not None


def is_llvm_validation_error(message: str) -> bool:
    return "LLVM Module verification failed" in message


class Report:
    def __init__(self, src: str, cmd: List[str], execution: subprocess.CompletedProcess, run_time: float):
        self.src = src
        self.cmd = cmd
        self.execution = execution
        self.run_time = run_time
        self.test_id = None

    @property
    def successful(self):
        return self.execution.returncode == 0

    @property
    def missing_feature(self):
        match = check_is_missing_feature(self.stderr)
        if match:
            return {
                "file": match.group(1),
                "line": match.group(2),
                "column": match.group(3)
            }

    @property
    def has_panic(self):
        match = check_is_other_panic(self.stderr)
        if match:
            return {
                "file": match.group(2),
                "line": match.group(3),
                "column": match.group(4),
                "message": match.group(1)
            }

    @property
    def stderr(self):
        return escape_ansi(self.execution.stderr.decode("utf-8"))

    @property
    def error_class(self):
        if self.successful:
            return "none"
        if is_circom_error(self.stderr):
            return "circom"
        if self.missing_feature is not None:
            return "todo!"
        if self.has_panic is not None:
            return "panic!"
        if is_llvm_validation_error(self.stderr):
            return "invalid llvm ir"
        return "other"

    def to_dict(self) -> dict:
        return {
            'src': self.src,
            'cmd': self.cmd,
            'return_code': self.execution.returncode,
            'successful': self.successful,
            'stdout': escape_ansi(self.execution.stdout.decode("utf-8")),
            'stderr': self.stderr,
            'run_time': self.run_time,
            'missing_feature': self.missing_feature,
            'error_class': self.error_class,
            'panicked': self.has_panic,
            'test_id': self.test_id
        }


def escape_ansi(line: str) -> str:
    ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -\/]*[@-~]')
    return ansi_escape.sub('', line)


def run_test(src: str, circom: str, debug: bool, cwd: str, libs_path: str) -> Report:
    src = os.path.realpath(src)
    tmp = TemporaryDirectory()
    cmd = [
        circom,
        '--llvm',
        '-o', tmp.name,
        '-l', libs_path,
        src
    ]
    start = time.time()
    execution = subprocess.run(cmd, capture_output=True, cwd=cwd)
    end = time.time()
    if debug:
        print("Source file:", src)
        print("CMD:", ' '.join(cmd))
        if execution.returncode == 0:
            print("Success!")
        else:
            print("Failure!")
        # if execution.stdout:
        #     print("Circom stdout:\n", escape_ansi(execution.stdout.decode("utf-8")))
        if execution.stderr:
            print("Circom stderr:\n", execution.stderr.decode("utf-8"))
        print("Execution time in seconds:", end - start)

    return Report(src, cmd, execution, end - start)


def evaluate_test(test_path: str, circom: str, debug: bool, libs_path: str) -> List[Report]:
    reports = []
    with open(test_path) as f:
        test_data = json.load(f)
    test_cwd = Path(test_path).parent
    for n, test in enumerate(test_data['tests']):
        main_circom_file = test_cwd.joinpath(test['main'])
        report = run_test(str(main_circom_file), circom, debug, str(test_cwd), libs_path)
        report.test_id = f"{test_data['id']}_{n}"
        reports.append(report)
    return reports


@click.command()
@click.option('--src', help='Path where the benchmark is located.')
@click.option('--out', help='Location of the output report.')
@click.option('--circom', help="Optional path to circom binary", default=os.path.realpath("target/release/circom"))
@click.option('--debug', help="Print debug information", is_flag=True)
def main(src, out, circom, debug):
    reports = []
    src = Path(src)
    tests = glob.glob(str(src.joinpath(GLOB)), recursive=True)
    for test in tests:
        reports.extend(evaluate_test(test, circom, debug, str(src.joinpath("tests/libs"))))

    with open(out, 'w') as out_csv:
        print('test_id,successful,error_class,message,file,line,column', file=out_csv)
        for report in reports:
            report = report.to_dict()
            if report['successful']:
                print(report['test_id'], report['successful'], '', '', '', '', '', sep=',', file=out_csv)
            else:
                if report['panicked']:
                    print(report['test_id'], report['successful'], report['error_class'],
                          f"\"{report['panicked']['message']}\"",
                          report['panicked']['file'], report['panicked']['line'], report['panicked']['column'], sep=',',
                          file=out_csv)
                else:
                    print(report['test_id'], report['successful'], report['error_class'], '', '', '', '', sep=',',
                          file=out_csv)


if __name__ == "__main__":
    main()
