import math
import psutil
import platform
import webbrowser
from datetime import datetime

def generate_html_specs():
    html_specs = """
    <table style='border: 1px solid black'>
        <tr>
            <th style='border: 1px solid black'>Architecture</th>
            <th style='border: 1px solid black'>""" + platform.machine() + """</th>
        </tr>

        <tr>
            <th style='border: 1px solid black'>Platform</th>
            <th style='border: 1px solid black'>""" + platform.system() + """</th>
        </tr>

        <tr>
            <th style='border: 1px solid black'>CPU Cores</th>
            <th style='border: 1px solid black'>""" + str(psutil.cpu_count()) + """</th>
        </tr>

        <tr>
            <th style='border: 1px solid black'>RAM</th>
            <th style='border: 1px solid black'>""" + str(round(psutil.virtual_memory().total / (1024.0 **3))) + """GB</th>
        </tr>
    </table>
    """
    
    return html_specs

def generate_queries_table(all_benches):
    html_queries = """
        <h3 style="font-weight: normal;">
            - <b>query index:</b> just the select queries numbered from 0 to N so that benchmark analysis is simplified.
            <br>
            - <b>select statement:</b> the SQL select statement used in the given benchmark
            <br>
            - <b>multi row plot params:</b> the exact parameters passed to the rust benchmark program
            <br>
            - <b>multi row plot:</b>the svg plot for the given benchmark, showing how the execution time increases as the number of table rows also increase.
            <br>
            - <b>callgrind plot params:</b> the exact parameters passed to the rust benchmark program during the callgrind profiling.
            <br>
            - <b>callgrind plot:</b> the callgrind svg tree generated out of the benchmark profiling.
            <br>
            - <b>callgring out file:</b> the exact valgrind out file generated for the given benchmark.
            <br>
            - <b>reference callgrind plot:</b> the svg callgrind tree generated in a previous benchmark,
            which is now being used as a comparison with the current benchmark.
        </h3>

        <br>

        <table style='border: 1px solid black'>
    <tr>
        <th style='border: 1px solid black'>Query Index</th>
        <th style='border: 1px solid black'>Select Statement</th>
        <th style='border: 1px solid black'>Multi Row Plot Params</th>
        <th style='border: 1px solid black'>Multi Row Plot</th>
        <th style='border: 1px solid black'>Callgrind Plot Params</th>
        <th style='border: 1px solid black'>Callgrind Plot</th>
        <th style='border: 1px solid black'>Valgrind Out File</th>
        <th style='border: 1px solid black'>Reference Callgrind Plot</th>
    </tr>
    """

    for count, bench in enumerate(all_benches):
        html_queries += """
        <tr>
            <th style='border: 1px solid black'>
            """ + str(count) + """
            </th>
            <th style='border: 1px solid black'>
            """ + bench.select_statement() + """
            </th>
            <th style='border: 1px solid black'>
                <a href='""" + bench.plot_bench_params_file() + """'>Params</a>
            </th>
            <th style='border: 1px solid black'>
                <a href='""" + bench.plot_bench_svg_file() + """'>Plot</a>
            </th>
            <th style='border: 1px solid black'>
                <a href='""" + bench.callgrind_params_file() + """'>Params</a>
            </th>
            <th style='border: 1px solid black'>
                <a href='""" + bench.callgrind_svg_file() + """'>Plot</a>
            </th>
            <th style='border: 1px solid black'>
                <a href='""" + bench.callgrind_out_file() + """'>File</a>
            </th>
            <th style='border: 1px solid black'>
                <a href='""" + bench.ref_callgrind_svg_file() + """'>Reference File</a>
            </th>
        </tr>
        """

    html_queries += """</table>
        <br>
    """

    return html_queries

def generate_summary_table(all_benches):
    html_summary_tables = """
        <h3 style="font-weight: normal;">
            - <b>query_index:</b> stands for the select query from table below. For instance, 
            query index 3 is the same one as the fourth row from the previous table, related to the query
            `select A,B from T where not (B = 1)`.
            <br>
            - <b>number of table rows:</b> is exactly what it says. The number of rows in the table used for the specified
            benchmark.
            <br>
            - <b>current execution time:</b> is the total time necessary to generate a proof and verify it in the specified VM
            (described below in the Specs section) and for the given benchmark settings.
            <br>
            - <b>current Throughput:</b> is simply the number of table rows processed for the given execution time, given
            in rows/minutes instead of rows/milliseconds. Note that this is considering the whole table length,
            not the result columns retrieved from the query.
            <br>
            - <b>reference execution time:</b> shows the execution time obtained in a previously executed benchmark,
            which is now being used as a comparison with the current one.
            <br>
            - <b>reference throughput:</b> is similar to the Current Throughput, but for the reference execution time.
            <br>
            - <b>status of current benchmark:</b> shows if the current benchmark is faster or slower than the reference benchmark data.
            If faster, then the message `improved` is shown. If slower, then the message `deteriorated` is shown.
            If their difference is negligible, then the message `not changed` is shown.
            - <b>speedup of current benchmark:<b> shows how much the current execution time improved compared to the reference benchmark.
        </h3>

        <br>
    
    <table style='border: 1px solid black'>
    <tr>
        <th style='border: 1px solid black'>Query Index</th>
        <th style='border: 1px solid black'>Number of Table Rows</th>
        <th style='border: 1px solid black'>Current Execution Time (ms)</th>
        <th style='border: 1px solid black'>Current Throughput (rows/min)</th>

        <th style='border: 1px solid black'>Reference Execution Time (ms)</th>
        <th style='border: 1px solid black'>Reference Throughput (rows/min)</th>
        <th style='border: 1px solid black'>Status of current Benchmark</th>
        <th style='border: 1px solid black'>Speedup of current Benchmark<br>(Current Execution Time / Reference Execution Time)</th>
    </tr>
    """

    for count_query, bench in enumerate(all_benches):
        for count_table, curr_table_length in enumerate(bench.table_lengths()):
            speedup = '?'
            exec_time = '?'
            throughput = '?'

            ref_exec_time = '?'
            ref_throughput = '?'
            ref_status = '?'

            if int(count_table) < len(bench.execution_times()):
                exec_time = bench.execution_times()[count_table]
                throughput = curr_table_length / float(exec_time) * 1e3 * 60
                throughput = '%.2g' % throughput

                if int(count_table) < len(bench.ref_execution_times()):
                    ref_exec_time = float(bench.ref_execution_times()[count_table])
                    ref_throughput = curr_table_length / float(bench.ref_execution_times()[count_table]) * 1e3 * 60
                    ref_throughput = '%.2g' % ref_throughput
                    speedup = ref_exec_time / exec_time

                    if math.fabs(1 - speedup) < 1e-3: ref_status = 'not changed'
                    elif float(ref_exec_time) < float(exec_time): ref_status = 'deteriorated'
                    else: ref_status = 'improved'

                    speedup = '%.2fx' % speedup

            html_summary_tables += """
                <tr>
                    <th style='border: 1px solid black'>""" + str(count_query) + """</th>
                    <th style='border: 1px solid black'>""" + str(curr_table_length) + """</th>
                    <th style='border: 1px solid black'>""" + str(exec_time) + """</th>
                    <th style='border: 1px solid black'>""" + str(throughput) + """</th>
                    <th style='border: 1px solid black'>""" + str(ref_exec_time) + """</th>
                    <th style='border: 1px solid black'>""" + str(ref_throughput) + """</th>
                    <th style='border: 1px solid black'>""" + str(ref_status) + """</th>
                    <th style='border: 1px solid black'>""" + str(speedup) + """</th>
                </tr>
            """

        if count_query + 1 < len(all_benches):
            html_summary_tables += """
                    <tr>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                        <th style='border: 1px solid black'>-</th>
                    </tr>
                """

    html_summary_tables += "</table>"

    return html_summary_tables

def get_introduction_text():
    intro = """
        <p>In order to spot bottlenecks in the proofs code, we need to establish a concise benchmark suite.
        While criterion is an excellent tool to achieve that, it poses some restrictions, such as
        automatically acquiring statistical data from the execution. Another approach is setting up an
        independent rust program to benchmark the proofs code and calling it from an external script,
        which would be responsible for gathering the correct statistical data. Our benchmark solution
        is following this last approach. Firstly, we use a rust program to
        benchmark our SQL-proof creation and verification. Then we use python scripts to aggregate
        various data from the executed benchmarks. Finally, we use those data to plot graphs and generate summary tables.</p>

        <p>During this process, we use multiple parameters to control the execution. For instance, we specify
        the exact select query we want to benchmark, which is given by the `query index`. We also specify
        the exact amount of table rows in each benchmark as well as the number of result columns for the
        query. From this process, we generate matplotlib graphs alongside callgrind benchmark files.
        Note that all those information can be found in the `Query Plot` table below. Besides, we also
        generate some summary data based on the information generated during the benchmarks. Those can be
        found in the `Query Statistics` table. As a highlight, we execute at least 5 times each benchmark
        so that results are not affected too much by outlier executions.</p>

        <p>Note that in addition to the benchmark data depicted, we also provide comparisons and links to
        a reference benchmark. Those are benchmarks that were generated in previous executions that are now
        used as comparisons against the current benchmark. For that, we also provide a simple comparison with
        this reference data in the last columns of the query statistics table (`Status`), showing if the
        current benchmark is faster than the reference one for the same select statement (`improved`),
        it's equivalent (`not changed`), or it's worst (`deteriorated`).</p>
    """

    return intro

def create_html_files(html_file, all_benches, statistics_data_file, all_plots_data_file, tar_tgz_data_file, open_html):
    html_summary_tables = generate_summary_table(all_benches)

    html_queries = generate_queries_table(all_benches)

    html_specs = generate_html_specs()

    with open(html_file, 'w') as f:
        curr_date = datetime.now().strftime('%m/%d/%Y %H:%M:%S')

        # the html code which will go in the file index.html
        html_template = """<html>
        <head>
        <title>Proofs Benchmarks</title>
        </head>
        <body>

        <h1> Proofs Benchmarks (""" + curr_date +  """) <h1>

        <hr>

        <h3 style="font-weight: normal;">""" + get_introduction_text() + """</h3>

        <hr>

        <h2> Query Plots (
            <a href='""" + all_plots_data_file + """'>all_plots.svg</a>,
            <a href='""" + tar_tgz_data_file + """'>all_benchmark_data.tgz</a>
        ): </h2>

        """ + html_queries + """

        <hr>

        <h2>
            Query Statistics (
                <a href='""" + statistics_data_file + """'>current benchmark data</a>,
                <a href='ref_""" + statistics_data_file + """'>reference benchmark data</a>
            ):
        </h2>

        """ + html_summary_tables + """

        <hr>

        <h2> System Specs: </h2>

        """ + html_specs + """

        <hr>
        
        </body>
        </html>
        """

        f.write(html_template)

    if open_html: webbrowser.open(html_file)
