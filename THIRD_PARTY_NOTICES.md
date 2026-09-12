# Third-party notices

Tuffite-owned code uses the root MIT OR Apache-2.0 LICENSE. Chromium-owned patch
regions and adapted Chromium sources keep their upstream notices; the Chromium
BSD terms are preserved in LICENSE.chromium. The Chromium checkout also has
its own LICENSE and third-party metadata under third_party/.

Framework packaging includes the Tuffite and Chromium licenses and generates
Chromium dependency notices for the selected GN build graph in runtime/licenses/.
They are covered by the Distribution file hashes and copied into application
packages. Do not strip that directory when redistributing a native package.

Application authors must also include notices for their Rust and frontend
dependencies, resources, and any other bundled third-party material. Tuffite's
Framework notices do not cover arbitrary application dependencies.
