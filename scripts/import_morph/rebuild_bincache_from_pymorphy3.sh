#!/usr/bin/env bash
# Rebuild opencorpora.bincache from pymorphy3 dictionary.
# Requires: python3.11+ with pymorphy3 and pymorphy3-dicts-ru installed.
# Produces: data/lexicon/opencorpora.bincache (RLM2 format)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TSV="/tmp/opencorpora_pymorphy3_export.tsv"
CACHE="$PROJECT_ROOT/data/lexicon/opencorpora.bincache"

echo "Step 1/3: Exporting pymorphy3 dictionary to TSV..."
python3.11 -c "
import pymorphy3, csv, sys
morph = pymorphy3.MorphAnalyzer()
d = morph.dictionary
writer = csv.writer(open('$TSV', 'w', newline=''), delimiter='\t', lineterminator='\n')
writer.writerow(['form','lemma','pos','features','lemma_id','paradigm_id','source_id','stress'])
count = 0
for word, tag, normal_form, para_id, idx in d.iter_known_words():
    pos = str(tag.POS or 'Other')
    features = '|'.join(str(g) for g in tag.grammemes if str(g) != pos)
    writer.writerow([word, normal_form, pos, features, str(idx), str(para_id), 'opencorpora.via.pymorphy3', ''])
    count += 1
    if count % 500000 == 0:
        print(f'  {count}...', file=sys.stderr)
print(f'Exported {count} entries', file=sys.stderr)
"
echo "Step 2/3: Compiling TSV to RLM2 bincache..."
cargo run --manifest-path "$PROJECT_ROOT/Cargo.toml" -- compile-lexicon --input "$TSV" --output "$CACHE"
echo "Step 3/3: Cleanup temp TSV..."
rm -f "$TSV"
echo "Done. Cache at: $CACHE"
