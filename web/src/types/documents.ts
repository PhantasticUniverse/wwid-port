/** Mirrors Rust InstrumentRaw (serde JSON). */
export interface InstrumentData {
  name: string;
  description?: string;
  lengthType: LengthType;
  mouthpiece: MouthpieceData;
  borePoint: BorePointData[];
  hole: HoleData[];
  termination: TerminationData;
}

export type LengthType = "in" | "cm" | "mm" | "m" | "ft";

export interface MouthpieceData {
  position: number;
  beta?: number;
  fipple?: FippleData;
  embouchureHole?: EmbouchureHoleData;
  singleReed?: { alpha: number };
  doubleReed?: { alpha: number; crowFreq: number };
  lipReed?: { alpha: number };
}

export interface FippleData {
  windowLength: number;
  windowWidth: number;
  fippleFactor?: number;
  windowHeight?: number;
  windwayLength?: number;
  windwayHeight?: number;
}

export interface EmbouchureHoleData {
  length: number;
  width: number;
  height: number;
  airstreamLength: number;
  airstreamHeight: number;
}

export interface BorePointData {
  name?: string;
  borePosition: number;
  boreDiameter: number;
}

export interface HoleData {
  name?: string;
  borePosition: number;
  diameter: number;
  height: number;
  innerCurvatureRadius?: number;
  key?: KeyData;
}

export interface KeyData {
  diameter: number;
  holeDiameter: number;
  height: number;
  thickness: number;
  wallThickness: number;
  chimneyHeight: number;
}

export interface TerminationData {
  flangeDiameter: number;
}

/** Mirrors Rust Tuning (serde JSON). */
export interface TuningData {
  name: string;
  comment?: string;
  numberOfHoles: number;
  fingering: FingeringData[];
}

export interface FingeringData {
  note: NoteData;
  openHole: boolean[];
  openEnd?: boolean;
  optimizationWeight?: number;
}

export interface NoteData {
  name: string;
  frequency?: number;
  frequencyMin?: number;
  frequencyMax?: number;
}

/** Mirrors Rust Constraints (serde JSON). */
export interface ConstraintsData {
  constraintsName: string;
  objectiveDisplayName: string;
  objectiveFunctionName: string;
  numberOfHoles: number;
  constraint: ConstraintData[];
}

export interface ConstraintData {
  displayName: string;
  category: string;
  type: ConstraintType;
  lowerBound?: number;
  upperBound?: number;
}

export type ConstraintType = "DIMENSIONAL" | "DIMENSIONLESS" | "INTEGER" | "BOOLEAN";
