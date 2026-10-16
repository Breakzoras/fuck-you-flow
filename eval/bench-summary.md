| Model | GPU | Load ms | Greek WER | Greek CER | English WER | Term acc. | Halluc. (silence+noise) | P50 ms | Max ms |
|---|---|---|---|---|---|---|---|---|---|
| large-v3-q5_0 | yes | 1541 | 0.182 | 0.144 | 0.212 | 0.833 | 2/2 | 783 | 1238 |
| large-v3-turbo-q5_0 | yes | 1030 | 0.250 | 0.171 | 0.212 | 0.5 | 2/2 | 304 | 397 |
| medium-q5_0 | yes | 1022 | 0.219 | 0.149 | 0.212 | 0.833 | 2/2 | 560 | 863 |

## On the processor, 12 October 2026

Same corpus (the 12 Greek and mixed clips), `--no-gpu`, 3 threads (a 4-core machine leaving one core free), beam 3 as the app uses. Run with a one-off script outside this folder, so `bench.py` rewriting this file will drop this section; the numbers live on in the comment on the small model in `src-tauri/src/models.rs`. Speed is from a Ryzen 7; an older 4-core Intel will be slower.

| Model | Greek WER | Seconds per second of speech | P50 ms (clips of 4.3 to 6.3 s) |
|---|---|---|---|
| small-q5_1 | 0.287 | 1.06 | 5316 |
| medium-q5_0 | 0.218 | 2.69 | 13445 |
| large-v3-turbo-q5_0 | 0.255 | 4.09 | 20621 |
