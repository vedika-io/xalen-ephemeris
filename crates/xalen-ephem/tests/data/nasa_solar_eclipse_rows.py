import re
import urllib.request

import swisseph as swe

CATALOG = "https://eclipse.gsfc.nasa.gov/SEcat5/SE{}.html"
PAGES = ("1901-2000", "2001-2100")
DATES = {
    "1902 May 07", "1928 May 19", "1950 Mar 18", "1957 Apr 30", "1957 Oct 23",
    "1967 Nov 02", "2014 Apr 29", "2024 Apr 08", "2043 Apr 09", "2043 Oct 03",
}
MONTHS = {m: i + 1 for i, m in enumerate("Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split())}
KINDS = {"T": "Total", "A": "Annular", "H": "Hybrid", "P": "Partial"}
ROW = re.compile(
    r"\s*\d{5}\s+((\d{4}) (\w{3}) (\d\d))\s+(\d\d):(\d\d):(\d\d)\s+(-?\d+)"
    r"\s+\S+\s+\S+\s+(\S+)\s+\S+\s+(-?\d\.\d+)"
)

for page in PAGES:
    html = urllib.request.urlopen(CATALOG.format(page)).read().decode("latin-1")
    for line in html.splitlines():
        m = ROW.match(re.sub(r"<[^>]*>", "", line))
        if not m or m.group(1) not in DATES:
            continue
        _, y, mon, d, h, mi, s, dt, code, gamma = m.groups()
        hours = int(h) + int(mi) / 60 + int(s) / 3600
        jd_ut = swe.julday(int(y), MONTHS[mon], int(d), hours) - int(dt) / 86400
        print(f'    ("{y}-{MONTHS[mon]:02}-{d}", "{code}", {KINDS[code[0]]}, {jd_ut:.6f}, {gamma}),')
