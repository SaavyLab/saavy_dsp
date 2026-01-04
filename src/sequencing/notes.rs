/*
MIDI Note Constants
===================

These constants provide readable names for MIDI note numbers.
Middle C (C4) = MIDI note 60, which is the standard reference point.

Naming Convention:
- Natural notes: C4, D4, E4, etc.
- Sharps: Cs4 (C#4), Ds4 (D#4), etc.
- Flats: Db4, Eb4, etc. (aliases for the same MIDI notes as sharps)

Octave Range:
- C0 (MIDI 12) to B8 (MIDI 119)
- Octave -1 (MIDI 0-11) omitted as rarely used

The MIDI formula: note_number = 12 * (octave + 1) + semitone
Where semitone: C=0, C#=1, D=2, D#=3, E=4, F=5, F#=6, G=7, G#=8, A=9, A#=10, B=11

Example usage:
  pattern!(4/4 => [C4, E4, G4, C5])  // C major arpeggio
  pattern!(4/4 => [A3, Cs4, E4])     // A major chord
*/

/// Rest - indicates silence for that slot
pub const REST: Option<u8> = None;

// Octave 0
pub const C0: u8 = 12;
pub const Cs0: u8 = 13;
pub const Db0: u8 = 13;
pub const D0: u8 = 14;
pub const Ds0: u8 = 15;
pub const Eb0: u8 = 15;
pub const E0: u8 = 16;
pub const F0: u8 = 17;
pub const Fs0: u8 = 18;
pub const Gb0: u8 = 18;
pub const G0: u8 = 19;
pub const Gs0: u8 = 20;
pub const Ab0: u8 = 20;
pub const A0: u8 = 21;
pub const As0: u8 = 22;
pub const Bb0: u8 = 22;
pub const B0: u8 = 23;

// Octave 1
pub const C1: u8 = 24;
pub const Cs1: u8 = 25;
pub const Db1: u8 = 25;
pub const D1: u8 = 26;
pub const Ds1: u8 = 27;
pub const Eb1: u8 = 27;
pub const E1: u8 = 28;
pub const F1: u8 = 29;
pub const Fs1: u8 = 30;
pub const Gb1: u8 = 30;
pub const G1: u8 = 31;
pub const Gs1: u8 = 32;
pub const Ab1: u8 = 32;
pub const A1: u8 = 33;
pub const As1: u8 = 34;
pub const Bb1: u8 = 34;
pub const B1: u8 = 35;

// Octave 2
pub const C2: u8 = 36;
pub const Cs2: u8 = 37;
pub const Db2: u8 = 37;
pub const D2: u8 = 38;
pub const Ds2: u8 = 39;
pub const Eb2: u8 = 39;
pub const E2: u8 = 40;
pub const F2: u8 = 41;
pub const Fs2: u8 = 42;
pub const Gb2: u8 = 42;
pub const G2: u8 = 43;
pub const Gs2: u8 = 44;
pub const Ab2: u8 = 44;
pub const A2: u8 = 45;
pub const As2: u8 = 46;
pub const Bb2: u8 = 46;
pub const B2: u8 = 47;

// Octave 3
pub const C3: u8 = 48;
pub const Cs3: u8 = 49;
pub const Db3: u8 = 49;
pub const D3: u8 = 50;
pub const Ds3: u8 = 51;
pub const Eb3: u8 = 51;
pub const E3: u8 = 52;
pub const F3: u8 = 53;
pub const Fs3: u8 = 54;
pub const Gb3: u8 = 54;
pub const G3: u8 = 55;
pub const Gs3: u8 = 56;
pub const Ab3: u8 = 56;
pub const A3: u8 = 57;
pub const As3: u8 = 58;
pub const Bb3: u8 = 58;
pub const B3: u8 = 59;

// Octave 4 (Middle C octave)
pub const C4: u8 = 60;
pub const Cs4: u8 = 61;
pub const Db4: u8 = 61;
pub const D4: u8 = 62;
pub const Ds4: u8 = 63;
pub const Eb4: u8 = 63;
pub const E4: u8 = 64;
pub const F4: u8 = 65;
pub const Fs4: u8 = 66;
pub const Gb4: u8 = 66;
pub const G4: u8 = 67;
pub const Gs4: u8 = 68;
pub const Ab4: u8 = 68;
pub const A4: u8 = 69; // A440 tuning reference
pub const As4: u8 = 70;
pub const Bb4: u8 = 70;
pub const B4: u8 = 71;

// Octave 5
pub const C5: u8 = 72;
pub const Cs5: u8 = 73;
pub const Db5: u8 = 73;
pub const D5: u8 = 74;
pub const Ds5: u8 = 75;
pub const Eb5: u8 = 75;
pub const E5: u8 = 76;
pub const F5: u8 = 77;
pub const Fs5: u8 = 78;
pub const Gb5: u8 = 78;
pub const G5: u8 = 79;
pub const Gs5: u8 = 80;
pub const Ab5: u8 = 80;
pub const A5: u8 = 81;
pub const As5: u8 = 82;
pub const Bb5: u8 = 82;
pub const B5: u8 = 83;

// Octave 6
pub const C6: u8 = 84;
pub const Cs6: u8 = 85;
pub const Db6: u8 = 85;
pub const D6: u8 = 86;
pub const Ds6: u8 = 87;
pub const Eb6: u8 = 87;
pub const E6: u8 = 88;
pub const F6: u8 = 89;
pub const Fs6: u8 = 90;
pub const Gb6: u8 = 90;
pub const G6: u8 = 91;
pub const Gs6: u8 = 92;
pub const Ab6: u8 = 92;
pub const A6: u8 = 93;
pub const As6: u8 = 94;
pub const Bb6: u8 = 94;
pub const B6: u8 = 95;

// Octave 7
pub const C7: u8 = 96;
pub const Cs7: u8 = 97;
pub const Db7: u8 = 97;
pub const D7: u8 = 98;
pub const Ds7: u8 = 99;
pub const Eb7: u8 = 99;
pub const E7: u8 = 100;
pub const F7: u8 = 101;
pub const Fs7: u8 = 102;
pub const Gb7: u8 = 102;
pub const G7: u8 = 103;
pub const Gs7: u8 = 104;
pub const Ab7: u8 = 104;
pub const A7: u8 = 105;
pub const As7: u8 = 106;
pub const Bb7: u8 = 106;
pub const B7: u8 = 107;

// Octave 8
pub const C8: u8 = 108;
pub const Cs8: u8 = 109;
pub const Db8: u8 = 109;
pub const D8: u8 = 110;
pub const Ds8: u8 = 111;
pub const Eb8: u8 = 111;
pub const E8: u8 = 112;
pub const F8: u8 = 113;
pub const Fs8: u8 = 114;
pub const Gb8: u8 = 114;
pub const G8: u8 = 115;
pub const Gs8: u8 = 116;
pub const Ab8: u8 = 116;
pub const A8: u8 = 117;
pub const As8: u8 = 118;
pub const Bb8: u8 = 118;
pub const B8: u8 = 119;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn middle_c_is_60() {
        assert_eq!(C4, 60);
    }

    #[test]
    fn a440_is_69() {
        assert_eq!(A4, 69);
    }

    #[test]
    fn octaves_are_12_apart() {
        assert_eq!(C5 - C4, 12);
        assert_eq!(C4 - C3, 12);
        assert_eq!(A5 - A4, 12);
    }

    #[test]
    fn sharps_and_flats_are_equal() {
        assert_eq!(Cs4, Db4);
        assert_eq!(Fs4, Gb4);
        assert_eq!(As4, Bb4);
    }

    #[test]
    fn chromatic_scale() {
        // C4 chromatic scale
        assert_eq!(C4, 60);
        assert_eq!(Cs4, 61);
        assert_eq!(D4, 62);
        assert_eq!(Ds4, 63);
        assert_eq!(E4, 64);
        assert_eq!(F4, 65);
        assert_eq!(Fs4, 66);
        assert_eq!(G4, 67);
        assert_eq!(Gs4, 68);
        assert_eq!(A4, 69);
        assert_eq!(As4, 70);
        assert_eq!(B4, 71);
    }
}
