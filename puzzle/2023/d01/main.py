input = open("input").read().splitlines()
count = 0
numbers = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]
for line in input:
    out = []
    i = 0
    s = ""
    while i < len(line):
        c = line[i]
        if c.isdigit():
            if s:
                for j, n in enumerate(numbers):
                    if n in s:
                        out.append(str(j + 1))
                s = ""
            out.append(c)
        else:
            s += c
        i += 1
    count += int(out[0] + out[-1])


print(count)
