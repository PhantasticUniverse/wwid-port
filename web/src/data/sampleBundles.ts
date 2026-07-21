export type SampleStudy = "NAF" | "Whistle" | "Flute" | "Reed";

export interface SampleFile {
  kind: "Instrument" | "Tuning" | "Constraints";
  label: string;
  path: string;
}

export interface SampleBundle {
  id: string;
  study: SampleStudy;
  title: string;
  description: string;
  files: SampleFile[];
}

export const SAMPLE_BUNDLES: SampleBundle[] = [
  {
    id: "naf-fsharp-starter",
    study: "NAF",
    title: "NAF F#4 Starter",
    description: "Six-hole Native American flute starter with chromatic tuning and hole-position constraints.",
    files: [
      { kind: "Instrument", label: '0.625" bore starter', path: "/samples/NafStudy/0.625-bore_6-hole_NAF_starter.xml" },
      { kind: "Tuning", label: "F#4 chromatic tuning", path: "/samples/NafStudy/Fsharp4_ET_6-hole_NAF_chromatic_tuning.xml" },
      { kind: "Constraints", label: "Hole-from-top constraints", path: "/samples/NafStudy/NAF_HoleFromTop_constraints.xml" },
    ],
  },
  {
    id: "naf-a4-large-bore",
    study: "NAF",
    title: "NAF A4 Large Bore",
    description: "One-inch bore NAF sample with A4 equal-temperament chromatic tuning.",
    files: [
      { kind: "Instrument", label: '1.00" bore starter', path: "/samples/NafStudy/1.00-bore_6-hole_NAF_starter.xml" },
      { kind: "Tuning", label: "A4 chromatic tuning", path: "/samples/NafStudy/A4_ET_6-hole_NAF_chromatic_tuning.xml" },
      { kind: "Constraints", label: "Hole-from-top constraints", path: "/samples/NafStudy/NAF_HoleFromTop_constraints.xml" },
    ],
  },
  {
    id: "naf-fsharp-woodwind-chromatic",
    study: "NAF",
    title: "NAF F#4 Wood Wind Chromatic",
    description:
      "Author's Wood Wind (Edward Kort) style F#4 chromatic tuning; the alternate G5 (closed) fingering carries optimization weight 0 so it is excluded from the residual norm.",
    files: [
      { kind: "Instrument", label: '0.625" bore starter', path: "/samples/NafStudy/0.625-bore_6-hole_NAF_starter.xml" },
      { kind: "Tuning", label: "F#4 chromatic (Wood Wind, weight-0 alt G5)", path: "/samples/NafStudy/Fsharp4_ET_6-hole_NAF_chromatic_WoodWind_tuning.xml" },
      { kind: "Constraints", label: "Hole-from-top constraints", path: "/samples/NafStudy/NAF_HoleFromTop_constraints.xml" },
    ],
  },
  {
    id: "whistle-pvc-d5",
    study: "Whistle",
    title: "Whistle PVC D5",
    description: "Simple six-hole PVC whistle with equal-temperament high-D tuning.",
    files: [
      { kind: "Instrument", label: "Sample PVC Whistle", path: "/samples/WhistleStudy/SamplePVC-Whistle.xml" },
      { kind: "Tuning", label: "D5 equal tuning", path: "/samples/WhistleStudy/D5-Equal.xml" },
      { kind: "Constraints", label: "Whistle hole constraints", path: "/samples/WhistleStudy/Whistle_Hole_constraints.xml" },
    ],
  },
  {
    id: "whistle-feadog",
    study: "Whistle",
    title: "Feadog Mk1 Measurement",
    description: "Measured commercial whistle geometry and tuning for reference comparisons.",
    files: [
      { kind: "Instrument", label: "Feadog Mk1", path: "/samples/WhistleStudy/FeadogMk1.xml" },
      { kind: "Tuning", label: "Feadog Mk1 tuning", path: "/samples/WhistleStudy/FeadogMk1-tuning.xml" },
      { kind: "Constraints", label: "Whistle hole constraints", path: "/samples/WhistleStudy/Whistle_Hole_constraints.xml" },
    ],
  },
  {
    id: "flute-pvc-d4",
    study: "Flute",
    title: "PVC Flute D4",
    description: "Transverse PVC flute starter with six-hole D4 tuning.",
    files: [
      { kind: "Instrument", label: "Sample PVC Flute", path: "/samples/FluteStudy/SamplePVC-Flute.xml" },
      { kind: "Tuning", label: "D4 equal tuning", path: "/samples/FluteStudy/D4-Equal.xml" },
      { kind: "Constraints", label: "Flute hole constraints", path: "/samples/FluteStudy/Flute_Hole_constraints.xml" },
    ],
  },
  {
    id: "flute-fife",
    study: "Flute",
    title: "Measured Fife",
    description: "Bb fife geometry with measured tuning data.",
    files: [
      { kind: "Instrument", label: "Fife", path: "/samples/FluteStudy/fife.xml" },
      { kind: "Tuning", label: "Fife tuning", path: "/samples/FluteStudy/fife-tuning.xml" },
      { kind: "Constraints", label: "Flute hole constraints", path: "/samples/FluteStudy/Flute_Hole_constraints.xml" },
    ],
  },
  {
    id: "reed-chanter",
    study: "Reed",
    title: "Smallpipe Chanter",
    description: "Eight-hole reed chanter sample with closed fingering tuning.",
    files: [
      { kind: "Instrument", label: "Sample Chanter", path: "/samples/ReedStudy/SampleChanter.xml" },
      { kind: "Tuning", label: "A3 closed fingering", path: "/samples/ReedStudy/A3-ClosedFingering.xml" },
      { kind: "Constraints", label: "Reed hole constraints", path: "/samples/ReedStudy/Reed_Hole_constraints.xml" },
    ],
  },
  {
    id: "reed-didgeridoo",
    study: "Reed",
    title: "Didgeridoo D2-D3",
    description: "Zero-hole lip-reed didgeridoo sample for bore/tuning exploration.",
    files: [
      { kind: "Instrument", label: "Two-stage didgeridoo", path: "/samples/ReedStudy/Didgeridoo-2stage-D2-D3.xml" },
      { kind: "Tuning", label: "D2-D3 tuning", path: "/samples/ReedStudy/Didgeridoo-D2-D3-tuning.xml" },
    ],
  },
];
