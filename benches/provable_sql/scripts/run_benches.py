#!/usr/bin/python3
import argparse
from benchmark_executor import BenchmarkExecutor


def main():
    parser = argparse.ArgumentParser(
        description="Script for generating performance benchmarks"
    )

    parser.add_argument(
        "--force-build",
        default=False,
        type=int,
        help="Specify if benchmark binary should be compiled",
    )
    parser.add_argument(
        "--min-value",
        default=-3,
        type=int,
        help="Specify the minimum allowed value in the benchmark database",
    )
    parser.add_argument(
        "--max-value",
        default=3,
        type=int,
        help="Specify the maximum allowed value in the benchmark database",
    )
    parser.add_argument(
        "--num-table-columns",
        default=5,
        type=int,
        help="Specify the number of columns to be created in the benchmark database",
    )
    parser.add_argument(
        "--num-result-columns",
        default=2,
        type=int,
        help="Specify the number of columns to be retrieved in the benchmark query",
    )
    parser.add_argument(
        "--num-samples",
        default=5,
        type=int,
        help="Specify the number of times that each benchmark should execute",
    )
    parser.add_argument(
        "--generate-plots",
        default=False,
        type=int,
        help="Specify if matplotlib svg files should be generated",
    )
    parser.add_argument(
        "--generate-callgrind",
        default=False,
        type=int,
        help="Specify if callgrind svg files should be generated",
    )
    parser.add_argument(
        "--open-html",
        default=False,
        type=int,
        help="Specify if the generated HTML file should be opened in a browser",
    )
    parser.add_argument(
        "--output-dir",
        default="target/benches/provable_sql/",
        type=str,
        help="Path to where output files will be stored",
    )
    parser.add_argument(
        "--ref-statistics-dir",
        default="benches/provable_sql/data/",
        type=str,
        help="Path to where the reference benchmarks files are stored",
    )

    parser.add_argument(
        "--plot-table-lengths",
        nargs="+",
        default=[
            1,
            10,
            100,
            1000,
        ],
        type=int,
        help="List with the number of table lengths to be used in the plot benchmarks",
    )

    parser.add_argument(
        "--callgrind-table-length",
        default=500,
        type=int,
        help="Specify the table length to be used with the callgrind benchmark",
    )

    executor = BenchmarkExecutor(parser.parse_args())

    executor.run_benchmarks()


if __name__ == "__main__":
    main()
