// Statics that name other statics, in an order no def-hash numbering keeps:
// every reference static must still see its target declared (tcc rejected
// the generated C when a static was defined before one it names).
static A: [u8; 3] = [7, 8, 9];
static B: &[u8] = &A;
static C: [u8; 2] = [1, 2];
static D: &[u8] = &C;
static E: [i64; 2] = [3, 4];
static F: &[i64] = &E;
static G: [u16; 1] = [5];
static H: &[u16] = &G;
static I: (&[u8], &[i64]) = (&A, &E);
static J: (&[u16], u8) = (&G, 6);
static K: &str = "k";
static L: (&str, &[u8]) = (K, &C);

fn main() {
    println!("{:?} {:?} {:?} {:?}", B, D, F, H);
    println!("{:?} {:?} {:?}", I, J, L);
}
