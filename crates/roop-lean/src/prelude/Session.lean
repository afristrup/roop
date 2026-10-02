namespace Roop.Session

/-- A session behaviour with checkpoints. A choice is made by the party that
selects, when `selects` is set, and waited for by the party that offers. -/
inductive B where
  | done : B
  | choice : Bool → Bool → List (String × B) → B
  | mu : String → B → B
  | var : String → B

mutual
def B.beq : B → B → Bool
  | .done, .done => true
  | .var a, .var b => a == b
  | .mu a x, .mu b y => a == b && B.beq x y
  | .choice k c bs, .choice k' c' bs' => k == k' && c == c' && B.beqBranches bs bs'
  | _, _ => false
def B.beqBranches : List (String × B) → List (String × B) → Bool
  | [], [] => true
  | (l, b) :: r, (l', b') :: r' => l == l' && B.beq b b' && B.beqBranches r r'
  | _, _ => false
end

instance : BEq B := ⟨B.beq⟩

mutual
def subst : B → String → B → B
  | .done, _, _ => .done
  | .var y, x, w => if y == x then w else .var y
  | .mu y b, x, w => if y == x then .mu y b else .mu y (subst b x w)
  | .choice k c bs, x, w => .choice k c (substBranches bs x w)
def substBranches : List (String × B) → String → B → List (String × B)
  | [], _, _ => []
  | (l, b) :: rest, x, w => (l, subst b x w) :: substBranches rest x w
end

mutual
def dual : B → B
  | .done => .done
  | .var x => .var x
  | .mu x b => .mu x (dual b)
  | .choice k c bs => .choice (!k) c (dualBranches bs)
def dualBranches : List (String × B) → List (String × B)
  | [] => []
  | (l, b) :: rest => (l, dual b) :: dualBranches rest
end

/-- Unrolls a recursion until a choice or the end shows. -/
def unfold : Nat → B → B
  | 0, _ => .done
  | n + 1, .mu x b => unfold n (subst b x (.mu x b))
  | _, b => b

abbrev Cfg := Option B × B × Option B × B

def mark (past : Option B) : B → Option B
  | b@(.choice _ true _) => some b
  | _ => past

/-- The paper's system for checkpoint compliance, run as a search that visits
every configuration once. -/
def holds : Nat → List Cfg → Option B → B → Option B → B → Option (List Cfg)
  | 0, _, _, _, _, _ => none
  | fuel + 1, seen, dp, c0, gp, s0 =>
    let c := unfold 64 c0
    let s := unfold 64 s0
    let cfg : Cfg := (dp, c, gp, s)
    if seen.contains cfg then some seen else
    let seen := cfg :: seen
    let rollback : List Cfg → Option (List Cfg) := fun seen =>
      match dp, gp with
      | none, none => some seen
      | some d, some g => holds fuel seen none d none g
      | _, _ => none
    match c, s with
    | .done, _ => rollback seen
    | .choice ks _ bc, .choice ss _ bs =>
      if ks == ss then none else
      let sel := if ks then bc else bs
      let off := if ks then bs else bc
      if !(sel.all fun p => off.any fun q => q.1 == p.1) then none else
      let dp' := mark dp c
      let gp' := mark gp s
      let walked := sel.foldlM (fun seen p =>
        match off.find? (fun q => q.1 == p.1) with
        | some q =>
          if ks then holds fuel seen dp' p.2 gp' q.2 else holds fuel seen dp' q.2 gp' p.2
        | none => none) seen
      walked.bind rollback
    | _, _ => none

def compliant (client server : B) : Bool := (holds 4096 [] none client none server).isSome

end Roop.Session
