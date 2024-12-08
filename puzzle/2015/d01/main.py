count = 0
with open("input") as f:
    for c in f.read().strip():
        if c == "(":
            count += 1
        elif c == ")":
            count -= 1
print(count)
