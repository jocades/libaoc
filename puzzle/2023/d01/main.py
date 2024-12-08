input = open("input").read().splitlines()
count = 0
for line in input:
    n = []
    for c in line:
        if len(n) == 2:
            break
        if c.isdigit():
            n.append(c)
    count += int("".join(n))


print(count)
