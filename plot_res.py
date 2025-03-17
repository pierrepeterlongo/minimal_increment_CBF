import re
import matplotlib.pyplot as plt
import sys
# Initialize lists to store extracted values
n_elements = []
overestimation_mean_opt = []
overestimation_mean_non_opt = []
insertion_time_opt = []
insertion_time_non_opt = []

# Regular expressions to extract data
n_elements_re = re.compile(r' Checking overestimation rate with n_elements = (\d+)')
insertion_time_opt_re = re.compile(r'Insertion time, with the optimization: ([\d\.]+)([µmns]+)')
insertion_time_non_opt_re = re.compile(r'Insertion time, without the optimization: ([\d\.]+)([µmns]+)')
overestimation_mean_opt_re = re.compile(r'Overestimation rate, optimized:.*?Overestimation mean: ([\d\.]+)', re.DOTALL)
overestimation_mean_non_opt_re = re.compile(r'Overestimation rate, non optimized:.*?Overestimation mean: ([\d\.]+)', re.DOTALL)

def convert_time_to_seconds(value, unit):
    conversions = {'s': 1, 'ms': 1e-3, 'µs': 1e-6, 'ns': 1e-9}
    return float(value) * conversions[unit]
# the input file is an argument
input_file = sys.argv[1]
# Read and parse the log file
with open(input_file, "r") as file:
    content = file.read()
    
    n_elements = [int(n) for n in n_elements_re.findall(content)]
    insertion_time_opt = [convert_time_to_seconds(v, u) for v, u in insertion_time_opt_re.findall(content)]
    insertion_time_non_opt = [convert_time_to_seconds(v, u) for v, u in insertion_time_non_opt_re.findall(content)]
    overestimation_mean_opt = [float(v) for v in overestimation_mean_opt_re.findall(content)]
    overestimation_mean_non_opt = [float(v) for v in overestimation_mean_non_opt_re.findall(content)]
    print(f"overestimation_mean_non_opt: {overestimation_mean_non_opt}")
    print(f"overestimation_mean_opt: {overestimation_mean_opt}")
    print(f"insertion_time_non_opt: {insertion_time_non_opt}")
    print(f"insertion_time_opt: {insertion_time_opt}")
    
    # Ensure all lists have the same length
    min_length = min(len(n_elements), len(overestimation_mean_opt), len(overestimation_mean_non_opt), len(insertion_time_opt), len(insertion_time_non_opt))
    n_elements = n_elements[:min_length]
    overestimation_mean_opt = overestimation_mean_opt[:min_length]
    overestimation_mean_non_opt = overestimation_mean_non_opt[:min_length]
    insertion_time_opt = insertion_time_opt[:min_length]
    insertion_time_non_opt = insertion_time_non_opt[:min_length]

# Debugging: Print extracted values
print("Extracted Data:")
print("n_elements:", n_elements)
print("Overestimation Mean (Optimized):", overestimation_mean_opt)
print("Overestimation Mean (Non-Optimized):", overestimation_mean_non_opt)
print("Insertion Time (Optimized):", insertion_time_opt)
print("Insertion Time (Non-Optimized):", insertion_time_non_opt)

# Plot the data
fig, ax1 = plt.subplots()

ax1.set_xlabel("Number of elements")
ax1.set_ylabel("Overestimation Mean", color='tab:blue')
ax1.plot(n_elements, overestimation_mean_opt, 'o-', color='tab:blue', label='Overestimation Mean (Optimized)')
ax1.plot(n_elements, overestimation_mean_non_opt, 's-', color='tab:cyan', label='Overestimation Mean (Non-Optimized)')
ax1.tick_params(axis='y', labelcolor='tab:blue')
# ax1.set_yscale('log')
ax1.legend(loc='upper left')

ax2 = ax1.twinx()
ax2.set_ylabel("Insertion Time (seconds)", color='tab:red')
ax2.plot(n_elements, insertion_time_opt, 'o--', color='tab:red', label='Insertion Time (Optimized)')
ax2.plot(n_elements, insertion_time_non_opt, 's--', color='tab:orange', label='Insertion Time (Non-Optimized)')
ax2.tick_params(axis='y', labelcolor='tab:red')
# ax2.set_yscale('log')
ax2.legend(loc='upper right')

plt.title("Overestimation Mean and Insertion Time vs Number of Elements")
plt.show()