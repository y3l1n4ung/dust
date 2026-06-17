import html
import re
import sys
from collections import Counter

path = sys.argv[1]
source = open(path, encoding="utf-8").read()

total = 0
rows = []
agg = Counter()

for title in re.findall(r"<title>(.*?)</title>", source):
    text = html.unescape(title)
    match = re.search(r" \((\d+) samples?, ([0-9.]+)%\)$", text)
    if not match:
        continue

    name = text[: match.start()]
    samples = int(match.group(1))
    percent = float(match.group(2))

    if name == "all":
        total = samples

    rows.append((samples, percent, name))
    agg[name] += samples

total = total or max((samples for samples, _, _ in rows), default=1)

print("Top individual frames")
for samples, percent, name in sorted(rows, reverse=True)[:40]:
    print(f"{samples:6d} {percent:6.2f}%  {name}")

print()
print("Top aggregated functions")
for name, samples in agg.most_common(40):
    print(f"{samples:6d} {samples / total * 100:6.2f}%  {name}")