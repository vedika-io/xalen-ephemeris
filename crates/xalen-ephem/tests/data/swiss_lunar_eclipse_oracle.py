import swisseph as swe

FLAGS = swe.FLG_MOSEPH
WINDOWS = [((2020, 1, 1), (2031, 1, 1))]
SINGLES = [(1901, 10, 27), (1939, 10, 28), (1950, 4, 2), (1958, 4, 4), (1988, 3, 3), (2015, 4, 4)]
CONTACTS = (6, 2, 4, 5, 3, 7)


def kind(flag):
    if flag & swe.ECL_TOTAL:
        return "Total"
    if flag & swe.ECL_PARTIAL:
        return "Partial"
    return "Penumbral"


def row(jd_search):
    flag, tret = swe.lun_eclipse_when(jd_search, FLAGS)
    _, attr = swe.lun_eclipse_how(tret[0], (0.0, 0.0, 0.0), FLAGS)
    y, m, d, _ = swe.revjul(tret[0])
    times = ", ".join(repr(tret[i]) for i in CONTACTS)
    return f'    ("{y:04}-{m:02}-{d:02}", {kind(flag)}, {tret[0]!r}, {attr[0]!r}, {attr[1]!r}, [{times}]),'


lines = []
for start, end in WINDOWS:
    jd, jd_end = swe.julday(*start, 0.0), swe.julday(*end, 0.0)
    while True:
        _, tret = swe.lun_eclipse_when(jd, FLAGS)
        if tret[0] >= jd_end:
            break
        lines.append(row(jd))
        jd = tret[0] + 20.0
for y, m, d in SINGLES:
    lines.append(row(swe.julday(y, m, d, 0.0) - 2.0))
print(f"swisseph {swe.version}")
print("\n".join(sorted(set(lines))))
