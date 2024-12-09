count = 0
out = None
with open("input") as f:
    for i, c in enumerate(f.read().strip()):
        if count == -1:
            out = i
            break
        if c == "(":
            count += 1
        elif c == ")":
            count -= 1

print(out)
