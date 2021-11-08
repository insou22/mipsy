#! /usr/bin/env bash

texts_failed=0

for test_file in test_files/success/*; do
    echo -n "Checking \"$test_file\"... "

    if $(echo $test_file | grep -q 'broken'); then
        echo 'Skipping broken file'
        continue
    fi

    mipsy_out=$( yes 3 2>/dev/null | ./target/debug/mipsy   "$test_file" 2>&1 | sed -E "/Loaded: .+/d")
    spim_out=$(  yes 3 2>/dev/null |                spim -f "$test_file" 2>&1 | sed -E "/Loaded: .+/d")

    if diff <(echo "$mipsy_out") <(echo "$spim_out") >/dev/null; then
        echo "PASSED"
    else
        echo "FAILED"
        echo "    mipsy_output: $(echo $mipsy_out)"
        echo ""
        echo "    spim_output:  $(echo $spim_out)"
        echo ""
        texts_failed=$((texts_failed + 1))
    fi
done

exit $((texts_failed == 0 ? 0 : 1))
