file_path = "output.txt"

with open(file_path, "r", encoding="utf-8") as f:
    data = f.read()
    target = data[:40391]
    print(target[-1000:])