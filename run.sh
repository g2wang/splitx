#!/bin/bash

cargo test > test/log

cat test/results/* > test/verify/_all
diff test/test.csv test/verify/_all > test/verify/diff
echo "diff added lines count:"
grep -c '^\d\+a\d\+$' test/verify/diff
echo "diff deleted lines count:"
grep -c '^\d\+d\d\+$' test/verify/diff
echo "last split file:"
ls -l test/results/ | tail -1
header=`head -1 test/test.csv`
echo "$header" > test/verify/combined.csv
grep -v '"Longitude (x)","Latitude (y)"' test/verify/_all >>  test/verify/combined.csv
echo "diff between combined and the original:"
diff test/verify/combined.csv test/test.csv

echo "wasted write (should not happen) count:"
grep -c 'wasted' test/log
