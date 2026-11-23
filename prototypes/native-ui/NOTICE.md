# Experimental dependency and source notices

YUR-005 · Feature `36be795f4590407eafd32028de2fc56e` · 2026-11-23.
This is an evaluation inventory, not redistribution clearance or product adoption.
The independently written Rust/Python fixture, renderer, guard and transfer code
do not copy Zed terminal or terminal_view sources. Synthetic text/pixels are
authored for this probe; no live conversation, account or remote image is used.

## Native toolkit and fixture dependencies

| Input | Exact source / integrity | Terms and retained notice |
| --- | --- | --- |
| GPUI Kit 0.11.0 | `longbridge/gpui-kit` revision `e97fe438129c5e4ccab504f2e6fafd9a4e396bba`; archive SHA-256 `5fb2bb331cc8f74d8fc7b571e636de9a0d14198d0a2ba98ba8a462c145e5b92c` | Native kit/base/component/component-macros declare Apache-2.0; [license](licenses/GPUI-KIT-APACHE-2.0.txt). Original headers remain in upstream dependencies. |
| GPUI snapshot family | Exact `gpui-pre =0.3.14` family in Cargo.lock; source metadata references Zed `f126950f9d3b62666f97437ae5d4ccac2272c4db` | Inspected native manifests/sources declare Apache-2.0. Published archives contain packaging adjustments; they are not asserted identical to the Zed checkout. Lock checksums bind downloaded bytes. |
| Kit bundled assets | Same Kit revision, `crates/assets` | Lucide ISC, Copyright (c) 2026 Lucide Icons and Contributors; Feather-derived icons MIT, Copyright (c) 2013-present Cole Bemis. [Complete bundled notice](licenses/KIT-ASSETS-LUCIDE-FEATHER.txt). |
| portable-pty 0.9.0 | Archive SHA-256 `b4a596a2b3d2752d94f51fac2d4a96737b8705dddd311a32b9af47211f08671e`; source metadata WezTerm `f8921727a11b9f8b073e8c24821d72fd41283500`, `pty` | MIT, Copyright (c) 2018 Wez Furlong. [License](licenses/PORTABLE-PTY-MIT.txt). |
| vt100 0.16.2 | Archive SHA-256 `054ff75fb8fa83e609e685106df4faeffdf3a735d3c74ebce97ec557d5d36fd9`; source metadata `eb66ffaf7d771c13303ef73b29f6f2a56fdacecf` | MIT, Copyright (c) 2016 Jesse Luehrs. [License](licenses/VT100-MIT.txt). |
| wl-clipboard-rs 0.9.4 | Archive SHA-256 `4d7888ccd4896447b2d14d3a9350a85df2aeb6f181e2e7a31349d104ac46cac1`; source metadata `f74da839b368e6a7c6d2f63acfe48d3d42474001` | MIT OR Apache-2.0; MIT option retained, Copyright (c) 2019 Ivan Molodetskikh. [License](licenses/WL-CLIPBOARD-RS-MIT.txt). |

The [target inventory](DEPENDENCIES.csv) records actual native normal/build
closures for Linux x86_64, macOS arm64 and Windows x64, with declared licenses,
source/checksum, resolved features and hashes of available root terms. Those
target resolutions are not target builds. The lock also contains optional/test
packages. Native system libraries, bundled third-party subfiles, fonts, shader
tools and final binary distribution obligations require further per-file review
before distribution. Empty root-terms cells do not mean notice-free material.

No upstream source is patched. Stock input Paste is intercepted before Kit's
uncapped read; text paste is deliberately unavailable. The bounded Linux Wayland
adapter reads native offers in an owned, deadline/resource-limited helper.
X11/macOS/Windows adapters and real compositor interoperability remain open.

## Separate host and rejected reuse

`gpui-shell` at the same Kit revision is a separate optional JavaScript host.
Its defaults enable Fast/webview; its runtime/transitive license closure is
incomplete. It is absent from this probe's graph. Neither host implementation nor
a plugin compatibility/security claim is made. Process isolation is not a sandbox.

Zed `crates/terminal` and `crates/terminal_view` at the snapshot reference declare
GPL-3.0-or-later and rely on application services. They were inspected as evidence
only and were not copied. GPUI's Apache terms do not cover those terminal crates.

Qt qtbase 6.12.1, revision `5af1a37e31a869cd6eb8a73d82e2c71a450e6634`,
`src/widgets/widgets/qdockwidget.cpp` provides narrow source evidence for floating
docks. Its header lists commercial / LGPL-3.0-only / GPL-2.0-only OR
GPL-3.0-only alternatives, Copyright (C) 2016 The Qt Company Ltd.
No Qt source or dependency is adopted; this is not a comparative benchmark.

## Independent development core and local build tools

These inputs are kept in ignored task-owned storage and are not redistributed
or installed by the prototype:

| Input | Pin / archive SHA-256 | Inspected root terms |
| --- | --- | --- |
| herdr-core 0.1.22 | `herdrdev/herdr-core` revision `300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e`; `bdc2c4edbe3d65f4dc0663af4dfd7dce8bbbc1d23d4c1b6291e030e335a92bcb` | Apache-2.0. Source/build scripts and explicit `serve`/endpoint contract inspected before build. |
| Ghostty vendor gitlink | `ghostty-org/ghostty` revision `1e5cdb865807abc990181efb8ebfa9122955cce2`; `3157b5a1ddc11040763a772a154b709ed7939a9626c3d1cd9207ebbab3fd2811` | MIT, Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors. Root license SHA-256 `386211873e5b7a02f663ae4d7adf96285999f91608f8f9f31fecfd0f4095e6f1`. Nested dependencies retain their own terms. |
| Official Zig 0.16.0, Linux x86_64 | `https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz`; 55,478,392 bytes; `70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00` matched official download index | MIT, Zig contributors; root license SHA-256 `5c537d6853e005298a285d508cff9ac7192cea23576c840d485b2b586a7ff177`. Bundled compiler/standard-library components have additional notices. |

HTTPS/source-tree/digest checks establish retrieved input bytes, not signed
release authenticity. Exact raw evidence, build exits and the local core binary
digest are in the ignored task proof. Only the verified independent build may be
staged; no installed/live-core fallback exists. All Cargo/Zig caches are owned
and command-scoped, and normal HOME is preserved.
