import re
from html_generator import *
from plot_generator import *
from system_utilities import *
from benchmark_setting import *
from statistics_generator import *

class BenchmarkExecutor:
    def __init__(self, args):
        self.args = args
        self.plots_data_file = 'plot_benchmark.svg'
        self.all_plots_data_file = 'all_plot_benchmark.svg'
        self.html_file = self.args.output_dir + '/index.html'
        self.statistics_data_file = 'statistics_benchmark_data.txt'
        self.tar_tgz_data_file = 'all_benchmark_data.tgz'
        self.table_lengths = self.args.plot_table_lengths.copy()
        self.binary_file = build_binary(self.args.force_build)
        self.exported_libs = "LD_LIBRARY_PATH='" + run_process('echo $(pwd)/$(dirname $(ci/find_lib.sh target/release/build/ libblitzar.so))/')[:-1] + "'"

    def build_bench_setting(self, query_idx, where_expr, num_result_cols, num_table_cols):
        return BenchmarkSetting(
            query_idx,
            self.args.output_dir,
            self.args.plot_table_lengths,
            self.args.callgrind_table_length,
            self.args.num_samples,
            self.args.min_value,
            self.args.max_value,
            num_table_cols,
            num_result_cols,
            where_expr,
            'plot_benchmark_params.txt',
            'plot_execution_times.txt',
            'plot_benchmark.svg',
            'callgrind_params.txt',
            'callgrind.svg',
            'callgrind.out',
            'callgrind.dot',
            'ref_callgrind.svg'
        )

    def process_all_queries(self):
        if self.args.num_table_columns < 4:
            print("Number of table columns must be at least 4")
            exit(1)

        if self.args.num_result_columns < 1:
            print("Number of result columns must be at least 1")
            exit(1)

        all_executions = []

        # these clauses specify each the `number of result columns`
        # `number of table columns` and the where expression to be
        # used in each experiment
        clauses = [
            (1, self.args.num_table_columns, 'B = 2'),
            (2, self.args.num_table_columns, 'B = 2'),
            (1, self.args.num_table_columns, 'not (B = 1)'),
            (2, self.args.num_table_columns, 'not (B = 1)'),
            (self.args.num_result_columns, self.args.num_table_columns, '(A = 2) and (B = 3)'),
            (self.args.num_result_columns, self.args.num_table_columns, '(A = 2) or (B = 3)'),
            (self.args.num_result_columns, self.args.num_table_columns, 'not ((A = 2) or (B = 3))'),
            (self.args.num_result_columns, self.args.num_table_columns, 'not ((A = 2) and (B = 3))'),
            (self.args.num_result_columns, self.args.num_table_columns, '((C = 0) or (B = 1)) and (not (A = -1))'),
            (self.args.num_result_columns, self.args.num_table_columns, '((C = 0) and (B = 1)) and (not (A = -1))'),
            (self.args.num_result_columns, self.args.num_table_columns, '((C = 0) and (B = 1)) or (not (A = -1))'),
            (self.args.num_result_columns, self.args.num_table_columns, '((C = 0) or (B = 1)) or (not (A = -1))'),
        ]

        for query_idx, clause in enumerate(clauses):
            # we need to build a benchmark setting, as this object will contain
            # all the relevant information about the current clause benchmark
            bench = self.build_bench_setting(query_idx, clause[2], clause[0], clause[1])

            self.run_callgrind_benchmark(bench)

            self.run_multi_rows_benchmark(bench)

            all_executions.append(bench)

        return all_executions

    def run_multi_rows_benchmark(self, bench):
        if self.args.generate_plots:
            execution_times = []

            for curr_table_length in bench.table_lengths():
                curr_params = bench.plot_params(curr_table_length)

                run_cmd = self.exported_libs + " " + self.binary_file + " " + curr_params

                # TODO: as GPU is now the default backend, 
                # this `run_process` will output
                # `WARN: Using pippenger cpu instead of naive gpu backend.`
                # when a GPU host is not available.
                # This message would not be shown with a GPU host.
                res = run_process(run_cmd)
                regex_match = re.search(r"\d+\.\d+seconds", res).group(0)
                print("Result: ", res, "; Regex: ", regex_match)
                mean_time = float(regex_match.replace('seconds', ''))

                execution_times.append(float(mean_time))
            
            bench.set_execution_times(execution_times)

            with open(self.args.output_dir + bench.plot_bench_params_file(), 'w') as fp:
                fp.write("\n".join(('--' + p) for p in curr_params.split('--')))

            with open(self.args.output_dir + bench.plot_execution_times_file(), 'w') as fp:
                fp.write("\n".join(str(ex) for ex in execution_times))

            print("Finished processing multi row benchmark query %d: %s" % (bench.idx(), bench.select_statement()))
        else:
            execution_file = self.args.output_dir + bench.plot_execution_times_file()

            if os.path.isfile(execution_file):
                with open(execution_file, 'r') as fp:
                    bench.set_execution_times([float(ex) for ex in fp.readlines()])

    def run_callgrind_benchmark(self, bench):
        if self.args.generate_callgrind:
            output_dir = self.args.output_dir

            curr_params = bench.callgrind_params()

            with open(output_dir + '/' + bench.callgrind_params_file(), 'w') as fp:
                fp.write("\n".join(('--' + p) for p in curr_params.split('--')))

            run_cmd = self.exported_libs + " valgrind --callgrind-out-file=" + output_dir + "/" + bench.callgrind_out_file() + " --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes --collect-atstart=no " + self.binary_file + " " + curr_params

            run_process(run_cmd)
            run_process('gprof2dot --format=callgrind --output=' + output_dir + '/' + bench.callgrind_dot_file() + ' ' + output_dir + '/' + bench.callgrind_out_file())
            run_process('dot -Tsvg ' + output_dir + '/' + bench.callgrind_dot_file() + ' -o ' + output_dir + '/' + bench.callgrind_svg_file())

            print("Finished processing callgrind benchmark query %d: %s" % (bench.idx(), bench.select_statement()))

    def run_benchmarks(self):
        # 1) process all benchmark rows
        all_benches = self.process_all_queries()
        
        # 2) save and copy necessary assets (note: this call modifies the all_benches elements)
        generate_and_copy_assets(
            self.args.output_dir,
            self.args.ref_statistics_dir,
            self.statistics_data_file,
            self.tar_tgz_data_file,
            all_benches
        )

        # 3) generate the matplotlib svg plots
        generate_all_plots_queries(self.args.output_dir, all_benches, self.all_plots_data_file)

        # 4) generate the html files pointing to the assets generated in the current benchmark
        create_html_files(
            self.html_file,
            all_benches,
            self.statistics_data_file,
            self.all_plots_data_file,
            self.tar_tgz_data_file,
            self.args.open_html
        )
