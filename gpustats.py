import gpustat

stats = gpustat.new_query()

gpu = stats[0]

output = f"""{gpu.name}
{gpu.memory_total}
{gpu.memory_used}
{gpu.temperature}
{gpu.utilization}
{gpu.fan_speed}"""

f = open("gpuinfo.txt", "w")
f.write(output)
f.close()