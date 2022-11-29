from system_utilities import *

class BenchmarkSetting:
    def __init__(self,
                idx,
                global_output_dir,
                table_lengths,
                callgrind_table_length,
                num_samples,
                min_par_val,
                max_par_val,
                num_table_cols,
                num_result_cols,
                where_expr,
                plot_bench_params_file,
                plot_execution_times_file,
                plot_bench_svg_file,
                plot_callgrind_params_file,
                plot_callgrind_svg_file,
                callgrind_out_file,
                callgrind_dot_file,
                ref_plot_callgrind_svg_file):

        self.idx_ = idx
        self.num_callgrind_samples_ = 1
        self.num_plot_samples_ = num_samples
        self.table_lengths_ = table_lengths.copy()
        self.callgrind_table_length_ = callgrind_table_length
        self.where_expr = where_expr
        self.execution_times_ = []
        self.ref_execution_times_ = []
        self.plot_bench_params_file_ = plot_bench_params_file
        self.plot_execution_times_file_ = plot_execution_times_file
        self.plot_bench_svg_file_ = plot_bench_svg_file
        self.callgrind_params_file_ = plot_callgrind_params_file
        self.callgrind_svg_file_ = plot_callgrind_svg_file
        self.callgrind_out_file_ = callgrind_out_file
        self.callgrind_dot_file_ = callgrind_dot_file
        self.ref_callgrind_svg_file_ = ref_plot_callgrind_svg_file
        self.select_statement_ = "select " + (','.join([chr(x + ord('A')) for x in range(num_result_cols)])) + ' from T where ' + where_expr
        self.min_value_table_ = min_par_val
        self.max_value_table_ = max_par_val
        self.num_table_columns_ = num_table_cols
        self.num_result_columns_ = num_result_cols
        self.base_params = "--min-value " + str(min_par_val)
        self.base_params += " --max-value " + str(max_par_val)
        self.base_params += " --num-columns " + str(num_table_cols)
        self.base_params += " --result-columns '" + (','.join([chr(x + ord('A')) for x in range(num_result_cols)])) + "'"

        run_process('mkdir -p ' + global_output_dir + '/' + self.base_dir())

    def idx(self):
        return self.idx_

    def select_statement(self):
        return self.select_statement_

    def ref_execution_times(self):
        return self.ref_execution_times_

    def set_ref_execution_times(self, ref_execution_times):
        self.ref_execution_times_ = ref_execution_times.copy()

    def base_dir(self):
        return 'query_' + str(self.idx())

    def table_lengths(self):
        return self.table_lengths_

    def callgrind_table_length(self):
        return self.callgrind_table_length_

    def execution_times(self):
        return self.execution_times_
    
    def set_execution_times(self, new_execution_times):
        self.execution_times_ = new_execution_times.copy()

    def plot_bench_params_file(self):
        return self.base_dir() + '/' + self.plot_bench_params_file_

    def plot_execution_times_file(self):
        return self.base_dir() + '/' + self.plot_execution_times_file_

    def plot_bench_svg_file(self):
        return self.base_dir() + '/' + self.plot_bench_svg_file_

    def callgrind_params_file(self):
        return self.base_dir() + '/' + self.callgrind_params_file_

    def callgrind_svg_file(self):
        return self.base_dir() + '/' + self.callgrind_svg_file_

    def callgrind_out_file(self):
        return self.base_dir() + '/' + self.callgrind_out_file_

    def callgrind_dot_file(self):
        return self.base_dir() + '/' + self.callgrind_dot_file_

    def ref_callgrind_svg_file(self):
        return self.base_dir() + '/' + self.ref_callgrind_svg_file_

    def plot_params(self, curr_table_length):
        return str(self.base_params) + " --where-expr '" + self.where_expr + "'" + " --num-samples " + str(self.num_plot_samples_) + " --table-length " + str(curr_table_length)

    def callgrind_params(self):
        return str(self.base_params) + " --where-expr '" + self.where_expr + "'" + " --num-samples " + str(self.num_callgrind_samples_) + " --table-length " + str(self.callgrind_table_length())

    def num_plot_samples(self):
        return self.num_plot_samples_

    def num_callgrind_samples(self):
        return self.num_callgrind_samples_

    def min_value_table(self):
        return self.min_value_table_

    def max_value_table(self):
        return self.max_value_table_
    
    def num_table_columns(self):
        return self.num_table_columns_
    
    def num_result_columns(self):
        return self.num_result_columns_
