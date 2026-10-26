# Native Danish analysis model

`danish-ddt.udpipe` is trained for this project from UD Danish DDT at commit
`5880293fa36da950fa833fe0b52cb7d21882c481`, using the UDPipe source packaged
in `udpipe-rs` 0.2.0. It is **CC BY-SA 4.0**, with attribution to the DDT
contributors and this project's training adaptation. It is not one of the
pretrained UD 2.5 models distributed under CC BY-NC-SA.

Upstream corpus: https://github.com/UniversalDependencies/UD_Danish-DDT

Corpus contributors credited upstream include Anders Johannsen, Héctor
Martínez Alonso and Barbara Plank; DDT originates from the Danish Dependency
Treebank work at Copenhagen Business School. See the pinned upstream README
in [DDT-README.md](DDT-README.md) for the full corpus history and references.
Exact source URLs and checksums are recorded in `training-sources.json`.

- Training: `da_ddt-ud-train.conllu`.
- Development/early stopping: `da_ddt-ud-dev.conllu`.
- The corpus's test split is not used for training or model selection.
- No blog text or linter examples are included in the training data.
- Tokenizer: UDPipe generic; tagger: 20 iterations; parser: 10 iterations.

Reproduce with `scripts/train-model.sh`. It checks the training-data SHA-256
and uses the UDPipe version locked by Cargo. It saves the resulting model
and log to `data/training/`; it does not silently replace the shipped model.
Floating-point/compiler differences may affect trained bytes and outputs.

The model is bundled into the release executable. COR and dictionaries are
installed separately by `danish setup`. Keep `LICENSE-DDT.txt` and this
attribution when redistributing the model or a binary containing it.

## Shipped model

Training completed 2026-10-26. Size: 7,762,948 bytes.
SHA-256: `a2e5e509e4717545a9b211e35c63671dbf16677ddde7e78800c3f6c3f885ccda`. See `manifest.json` and `training.log`.

Development-set selection chose tagger iteration 12 and parser iteration 10.
Parser development UAS: 77.24%; LAS: 73.17%, using generated tags. These
are dependency-parsing scores, **not linter accuracy** or an independent
evaluation of Danish proofreading.
