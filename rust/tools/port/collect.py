"""Decides which generated sources go into the repository, from the live check (`sk-check`) of each one.

  python collect.py --reports madara.json mangathemesia.json --check check.jsonl --out ../ported-sources.json

A source is published when its check says `ok`, or `cloudflare` (the PC's curl cannot pass a Cloudflare wall, the phone's
client can; the port is the same code as the extension). `dead` (the site is gone) and `broken` (the site answers but
the page no longer parses) stay out and are listed in `ported-sources.json` under `left_out`, with the reason.
"""

import argparse
import json


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reports", nargs="+", required=True)
    ap.add_argument("--check", nargs="+", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    ported = {}
    for r in a.reports:
        for x in json.load(open(r, encoding="utf-8"))["done"]:
            ported[x["dir"]] = x
    checks = {}
    for path in a.check:  # later files win: a source checked again replaces its older result
        for line in open(path, encoding="utf-8"):
            try:
                c = json.loads(line)
            except ValueError:
                continue
            checks[c["module"].split("sk_source_")[1].rsplit(".", 1)[0]] = c
    publish, left_out = [], []
    for d, x in sorted(ported.items()):
        c = checks.get(d, {})
        status = c.get("status", "unchecked")
        row = {"dir": d, "name": x["name"], "version": 1, "langs": ["en"], "theme": x["theme"], "check": status}
        if status in ("ok", "cloudflare"):
            publish.append(row)
        else:
            row["why"] = c.get("error", status)
            left_out.append(row)
    json.dump({"publish": publish, "left_out": left_out}, open(a.out, "w", encoding="utf-8"), indent=1)
    print(f"{len(publish)} to publish ({sum(1 for p in publish if p['check'] == 'cloudflare')} behind Cloudflare), {len(left_out)} left out")


if __name__ == "__main__":
    main()
