# weave on real tasks

weave trains reversible networks in Q12 fixed point and stores no activations. Until now it had
trained only XOR. This document is what happened when it trained a digit classifier and a
character model, side by side with torch, and which numerical problems that turned up. The code
is in `examples/weave/` (see its README to rerun any of it), and every number below is a line of
`examples/weave/results/*.jsonl`, printed by `examples/weave/report.py`.

## Summary

- On both tasks, trained through weave with sgd, momentum or Adam, the network reaches the test
  accuracy of the same network trained in doubles by torch's autograd (the "mirror"), within the
  spread across seeds. On digits at depth 8 that is 95.7 to 97.2 percent for weave and 94.8 to
  97.4 percent for the mirror. It does so with no activations stored: 8 to 32 MB of resident memory
  from 33 thousand to a million parameters, where autograd saves 1.3 to 66 MB of activations
  for the same batches, on top of a torch process of about 400 MB.
- Weave was not equal at the start. Running real tasks found seven numerical problems that
  trained toy networks hid, all fixed or reduced, each in a commit with the run that motivated
  it: sgd and momentum dropped most weight updates at small learning rates, Adam had no bias
  correction, the softmax exponential was 20 percent low at -5, the softmax seed lost small
  probabilities and Q12 adjoints underflowed with small weights (one fix, a loss scale), RMSNorm
  returned zero for a large hidden layer, and Adam's square root was slow. Open: arithmetic that
  overflows wraps silently and does not trap, learning rates cannot be finer than 1/4096, and
  weave diverges about as often as float at a high learning rate, with no warning when it does.
- The gap that does show is not fixed point. On characters the reversible network reaches 37
  to 40 percent against 41 to 44 percent for an ordinary MLP of the same size, and so does its
  float mirror.

## Setup

**Digits.** scikit-learn's `load_digits`: 1797 images of 8 by 8, pixels divided by 16, so the 64
pixels are the state of width 64. Shuffled by the seed, the last 449 are the test set and the
first 1344 train (a multiple of the batch). Softmax cross entropy over the first 10 outputs of `q`.

**Characters.** The text of `docs/reference.md` in lowercase; the 15 most common characters and one
slot for the rest make a vocabulary of 16, four characters of context are one-hot in a state of
width 64, and the 16 outputs are the next character. 16384 windows train and 10724 test.

**The reversible network.** One leapfrog layer, which takes the input in, then perceptron blocks
`Linear(64, 32), Tanh, Linear(32, 64)`: 8 layers is 33 thousand parameters. Deeper stacks add
blocks and scale the last weights of each by `sqrt(8 / depth)`, since a state that is the sum of
128 blocks at the default initialization does not train in float either. Attention, convolution
and RMSNorm blocks are tried in their own sections. `weave` is this network compiled by
`roop weave` and trained in Q12 by the program `roop weave --driver` writes.

**The baselines.** `mirror` is the same network as a torch module in doubles (`torch_mirror.py`
run on a batch, trained by autograd), so the function is the same, tanh Pade approximation
and all. `plain` is an ordinary MLP, 64-128-128-10, 26 thousand parameters, in float32 and in
doubles (the two are the same to the third decimal, so only one is shown below).

**Training.** Batches of 32 in the order of the data, no shuffling between epochs (weave does
not shuffle), the loss summed over the batch so that weave's summed gradients and torch's are
the same step at the same learning rate, torch Adam with `eps = 1e-3` (weave's 4/4096). Seeds 0 to
4 (digits, depth 8), 0 to 2 (characters and depth 32 and 64); depth 128 has one to four seeds
as stated. Each table row is the mean and standard deviation over its seeds, and the test
accuracy and cross entropy are of the final weights, evaluated in float on weave's
trained weights. That is checked against the compiled fixed point forward pass below. The
learning rates were chosen from a sweep on seed 0 (the table below) and used for all three
methods, except the two Adam rates, which are both shown. A run repeated gives the same numbers
to the digit: weave is deterministic, including its blow-ups.

**Timing is noisy.** The machine was shared with Lean runs and benchmarks (load average from 2
to 30 during these runs), so the time columns are CPU seconds of the training itself (the
compiled program for weave, the training loop for torch, whose threads can make CPU seconds exceed
wall time). Treat them as good to a factor of about 1.3.

## Results

### Digits, depth 8, 40 epochs, 5 seeds

All numbers after the fixes below. Test accuracy and cross entropy of the final weights, cross
entropy on the training set, the loss of the last epoch (half the squared error of the
probabilities, per sample), CPU seconds and peak resident memory. `x256` is weave with a loss scale
of 256 (see problem 5).

| method | optimizer | rate | test acc | test CE | train CE | cpu s | RSS MB | autograd saves MB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| plain MLP | sgd | 0.005 | 0.964 +- 0.009 | 0.144 +- 0.067 | 0.013 | 1.8 | 376 | 0.4 |
| mirror | sgd | 0.005 | 0.968 +- 0.014 | 0.141 +- 0.084 | 0.003 | 3.1 | 380 | 1.3 |
| weave | sgd | 0.005 | 0.967 +- 0.012 | 0.132 +- 0.073 | 0.004 | 0.9 | 7.5 | 0 |
| weave x256 | sgd | 0.005 | 0.967 +- 0.013 | 0.135 +- 0.076 | 0.003 | 1.4 | 7.5 | 0 |
| plain MLP | momentum | 0.001 | 0.968 +- 0.011 | 0.145 +- 0.080 | 0.003 | 1.8 | 376 | 0.4 |
| mirror | momentum | 0.001 | 0.974 +- 0.011 | 0.121 +- 0.073 | 0.001 | 3.2 | 380 | 1.3 |
| weave | momentum | 0.001 | 0.972 +- 0.011 | 0.115 +- 0.072 | 0.001 | 0.9 | 7.8 | 0 |
| weave x256 | momentum | 0.001 | 0.972 +- 0.008 | 0.122 +- 0.073 | 0.001 | 1.0 | 7.8 | 0 |
| plain MLP | adam | 0.001 | 0.965 +- 0.012 | 0.148 +- 0.079 | 0.010 | 2.0 | 376 | 0.4 |
| mirror | adam | 0.001 | 0.965 +- 0.013 | 0.151 +- 0.068 | 0.005 | 3.6 | 381 | 1.3 |
| weave | adam | 0.001 | 0.961 +- 0.011 | 0.151 +- 0.059 | 0.013 | 1.6 | 8.0 | 0 |
| weave x256 | adam | 0.001 | 0.963 +- 0.009 | 0.156 +- 0.059 | 0.008 | 2.9 | 8.1 | 0 |
| mirror | adam | 0.0005 | 0.948 +- 0.020 | 0.190 +- 0.111 | 0.037 | 3.6 | 381 | 1.3 |
| weave | adam | 0.0005 | 0.957 +- 0.012 | 0.151 +- 0.056 | 0.020 | 1.6 | 8.0 | 0 |
| weave x256 | adam | 0.0005 | 0.958 +- 0.011 | 0.153 +- 0.053 | 0.019 | 2.9 | 8.1 | 0 |

Weave is within one standard deviation of the mirror and the plain MLP in every row. The test
set is 449 images, so one image is 0.2 percent and the differences above are a few images. The
float mirror is the fair comparison for the arithmetic, the plain MLP for the architecture, and
on this task the reversible network and the plain MLP are the same.

The accuracy is evaluated in float on weights that weave trained in fixed point. Running the
449 test images through the compiled fixed point forward pass instead (`--fixed-eval`, seed 0)
gives the same accuracy, to the image: sgd 0.9755, momentum 0.9733, Adam 0.9599, in both.

Compute: weave's training took 0.9 CPU seconds with sgd for 40 epochs, against 1.8 for the
plain MLP and 3.1 for the mirror (which runs the network a sample at a time under `vmap`, so it
is the slow baseline). With Adam it is 1.6 against 2.0 and 3.6. These are one process on one
machine and the plain MLP is the honest comparison, so the claim is "the same order, a little
faster", not more. The loss scale makes Adam slower (2.9), since its square root runs on larger
numbers.

Loss curves (mean over 5 seeds, Adam 0.001, loss of the epoch per sample): epoch 1, 2, 3, 5, 10, 40
are 0.372, 0.149, 0.072, 0.036, 0.015, 0.002 for weave and 0.368, 0.142, 0.069, 0.034, 0.013, 0.002 for
the mirror. Before the Adam fix the weave curve was 0.207, 0.059, 0.032, 0.018, 0.005, 0.001,
which fell far faster than torch's (problem 2).

### Learning rate sweep, and what the carry fixed

Digits, depth 8, seed 0, 30 epochs. Test accuracy and training cross entropy. "before" is weave
as it was, "after" with the fixes (the carry in the optimizers, Adam's correction, the exponential).

| optimizer | rate | float mirror | weave before | weave after |
| --- | --- | --- | --- | --- |
| sgd | 0.0005 | 0.955 / 0.186 | 0.619 / 1.636 | 0.951 / 0.194 |
| sgd | 0.001 | 0.967 / 0.084 | 0.947 / 0.206 | 0.969 / 0.088 |
| sgd | 0.002 | 0.969 / 0.035 | 0.969 / 0.070 | 0.967 / 0.036 |
| sgd | 0.005 | 0.978 / 0.008 | 0.978 / 0.017 | 0.978 / 0.009 |
| sgd | 0.01 | 0.982 / 0.003 | 0.581 / 2.004 | 0.942 / 0.040 |
| momentum | 0.0002 | 0.971 / 0.033 | 0.962 / 0.122 | 0.976 / 0.027 |
| momentum | 0.0005 | 0.967 / 0.031 | 0.978 / 0.039 | 0.971 / 0.016 |
| momentum | 0.001 | 0.980 / 0.002 | 0.978 / 0.011 | 0.964 / 0.031 |
| adam | 0.0005 | 0.962 / 0.015 | 0.969 / 0.055 | 0.969 / 0.057 |
| adam | 0.001 | 0.933 / 0.177 | 0.976 / 0.017 | 0.960 / 0.028 |

Below the rate of 0.002 the old weave was visibly worse than float, and at 0.0005 it had
barely learned (0.619). The sgd 0.01 rows are one seed each of a run that is at the edge of
stability for all three (next section), so its row says little. The training cross entropy of
the old weave was about twice the float's at every rate that trained (0.017 against 0.008 at
0.005), and with the fixes it is within 15 percent.

### Depth: 8, 32, 64, 128 and 256 layers

Digits, 30 epochs, last weights of each block scaled by `sqrt(8 / depth)`, rates sgd 0.002,
momentum 0.0005, Adam 0.0005. Test accuracy, mean and standard deviation over the seeds.

| depth | seeds | optimizer | mirror | weave | autograd saves MB | weave RSS MB |
| --- | --- | --- | --- | --- | --- | --- |
| 32 | 3 | sgd | 0.965 +- 0.001 | 0.964 +- 0.004 | 2.9 | 9.9 |
| 32 | 3 | momentum | 0.963 +- 0.009 | 0.955 +- 0.013 | 2.9 | 10.9 |
| 32 | 3 | Adam | 0.964 +- 0.010 | 0.970 +- 0.006 | 2.9 | 12.0 |
| 64 | 3 | sgd | 0.965 +- 0.006 | 0.962 +- 0.011 | 5.0 | 13.0 |
| 64 | 3 | momentum | 0.939 +- 0.007 | 0.948 +- 0.021 | 5.0 | 15.1 |
| 64 | 3 | Adam | 0.970 +- 0.010 | 0.961 +- 0.015 | 5.0 | 17.2 |
| 128 | 1 | momentum | 0.953 | 0.951 | 9.2 | 23.4 |
| 128 | 1 | Adam | 0.973 | 0.964 | 9.2 | 27.7 |
| 128 | 4 | sgd | 0.974 +- 0.001 | 0.749 +- 0.429 | 9.2 | 19.3 |
| 128 | 4 | sgd, loss scale 256 | | 0.965 +- 0.009 | | |

Weave trains 128 layers as the float network does. The one exception is sgd at 128, where one
of four seeds of weave (seed 0) blew up in epoch 15 (loss 0.023 to 0.39 to 0.83 and no recovery,
which is the 0.749 average); the float mirror did not blow up in its four seeds, and weave with
the loss scale did not in four. That is one event, which could be luck (the float network
blows up at a similar rate, next section), but it is in the table.

Memory against depth and batch, one run each, digits, 2 epochs, sgd:

| depth | batch | parameters | torch process RSS MB | autograd saves MB | weave RSS MB | torch cpu s | weave cpu s |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 8 | 32 | 33504 | 380 | 1.3 | 7.5 | 2.0 | 0.05 |
| 8 | 128 | 33504 | 383 | 3.3 | 8.3 | 1.9 | 0.03 |
| 32 | 32 | 134112 | 390 | 2.8 | 9.9 | 2.1 | 0.17 |
| 32 | 128 | 134112 | 373 | 9.3 | 10.7 | 2.1 | 0.13 |
| 128 | 32 | 536544 | 431 | 8.9 | 19.3 | 3.5 | 0.63 |
| 128 | 128 | 536544 | 481 | 33.7 | 20.1 | 2.8 | 0.44 |
| 256 | 32 | 1073120 | 484 | 17.0 | 31.8 | 4.7 | 1.29 |
| 256 | 128 | 1073120 | 477 | 66.2 | 32.6 | 3.3 | 0.91 |

"Autograd saves" is the bytes of every tensor autograd keeps for the backward pass of one
batch, not counting the weights: it grows with depth times batch, from 1.3 to 66 MB. Weave keeps
none of it. What its memory does grow with is the parameters, each held with its gradient and
its carry (24 bytes per parameter for sgd, more for Adam), which is where 32 MB for a million
parameters comes from, and the same at batch 32 and 128. The torch process is about 380 MB before
it trains anything, so the saved activations are the honest comparison, and at this size they
are smaller than the parameter state weave holds. The saved bytes pass weave's total only where
depth times batch is large (depth 128, batch 128: 34 against 20 MB; depth 256: 66 against 33).
The memory advantage is real, grows without bound with depth times batch, and is small at the
sizes tried.

### Characters, 12 epochs, 3 seeds

Next-character accuracy over 16 symbols (the most common, a space, is 17 percent of the text), test
set 10724.

| method | optimizer | rate | test acc | test CE | train CE |
| --- | --- | --- | --- | --- | --- |
| plain MLP | sgd | 0.002 | 0.407 +- 0.002 | 1.851 +- 0.015 | 1.750 |
| mirror | sgd | 0.002 | 0.378 +- 0.010 | 1.956 +- 0.029 | 1.886 |
| weave | sgd | 0.002 | 0.372 +- 0.014 | 1.970 +- 0.050 | 1.894 |
| weave x256 | sgd | 0.002 | 0.372 +- 0.014 | 1.974 +- 0.051 | 1.900 |
| plain MLP | momentum | 0.0005 | 0.443 +- 0.009 | 1.745 +- 0.011 | 1.525 |
| mirror | momentum | 0.0005 | 0.377 +- 0.011 | 1.968 +- 0.045 | 1.875 |
| weave | momentum | 0.0005 | 0.376 +- 0.013 | 1.969 +- 0.040 | 1.871 |
| weave x256 | momentum | 0.0005 | 0.397 +- 0.009 | 1.935 +- 0.018 | 1.837 |
| plain MLP | adam | 0.0005 | 0.438 +- 0.004 | 1.765 +- 0.011 | 1.610 |
| mirror | adam | 0.0005 | 0.400 +- 0.011 | 1.888 +- 0.015 | 1.806 |
| weave | adam | 0.0005 | 0.400 +- 0.003 | 1.898 +- 0.020 | 1.809 |
| weave x256 | adam | 0.0005 | 0.401 +- 0.001 | 1.893 +- 0.020 | 1.804 |

Weave equals the float mirror to the second decimal in every row. Both fall short of the
plain MLP by 1 to 7 points, and the train cross entropy shows it is underfitting, not fixed point:
the reversible network has a hidden width of 32 against the plain network's 128, and a leapfrog
layer first. The task is one where the reversible architecture, not the arithmetic, is the limit.
Twelve epochs is underfit for all of them; the loss was still falling.

### Attention, convolution, other activations, batch size

Digits, depth 8, test accuracy / training cross entropy of the final weights, seed 0 unless noted.

| setting | float mirror | weave |
| --- | --- | --- |
| convolution blocks only, sgd 0.002 | 0.958 / 0.026 | 0.955 / 0.052 |
| convolution blocks only, Adam 0.0005 | 0.962 / 0.054 | 0.962 / 0.054 |
| mlp, attention, convolution cycled, Adam 0.0005 | 0.958 / 0.014 | 0.962 / 0.022 |
| mlp, attention, convolution cycled, sgd 0.002, 6 seeds | trained 2 of 6 | trained 3 of 6 (x256: 2 of 6) |
| attention blocks only, sgd 0.002 | NaN | stuck at 0.087 |
| relu, sgd 0.005 | 0.984 / 0.002 | 0.982 / 0.002 |
| gelu, sgd 0.005 | 0.987 / 0.003 | 0.976 / 0.014 |
| silu, sgd 0.005 | 0.982 / 0.004 | blew up (seed 0) |
| batch 8, sgd 0.005 | 0.973 / 0.010 | 0.973 / 0.011 |
| batch 16, sgd 0.005 | 0.973 / 0.010 | 0.971 / 0.011 |
| batch 32, sgd 0.005 | 0.980 / 0.009 | 0.978 / 0.010 |
| batch 64, sgd 0.005 | 0.984 / 0.009 | 0.940 / 0.083 |
| batch 128, sgd 0.005 | blew up | blew up |
| batch 8 / 16 / 32 / 64 / 128, Adam 0.001 | 0.976 / 0.976 / 0.960 / 0.962 / 0.962 | 0.964 / 0.969 / 0.958 / 0.967 / 0.973 |

The attention block here has no softmax (weave has no exponential in a block), so its output is
cubic in the weights and it is unstable in float too: the mirror reaches NaN and weave sits at
chance. Mixed with the other blocks it trains in only some seeds in both. A batch of 128 with
the same rate per sample is simply too large a step for sgd in float and in weave; Adam does not
care about the batch. Batch size has no effect on the arithmetic otherwise: the gradients of a
batch are the sum of the samples' to the bit.

### RMSNorm with large inputs

Digits with RMSNorm in each of the 7 perceptron blocks, sgd 0.002, test accuracy after 2 epochs
and the loss of the epoch, with the inputs multiplied by a scale:

| input scale | float mirror | weave before the fix | weave after |
| --- | --- | --- | --- |
| 1 (5 epochs) | 0.967, 0.027 | 0.971, 0.024 | 0.969, 0.026 |
| 8 | 0.931, 0.069 | 0.931, 0.069 | 0.931, 0.069 |
| 32 | 0.684, 0.350 | 0.492, 0.464 | 0.670, 0.371 |
| 128 | 0.143, 0.769 | 0.192, 0.775 | 0.145, 0.759 |

## Numerics: what was found

Each fixed problem is its own commit with the run that showed it.

**1. Sgd and momentum dropped most updates (fixed, `e03bfd7`).** A step is `lr * g` divided by
2^24, and the division truncates toward zero, so a step below one unit of the Q12 weight
vanished. On digits at depth 8, 55 percent of the updates at rate 0.005 and 98 percent at
0.0005 were below one unit (measured on the float gradients at the start; 90 percent at rate
0.005 after 30 epochs). That is the 0.619 at sgd 0.0005 in the table above. Now every
optimizer computes its step in Q24, and the part below a unit goes in a carry, one more tensor
of state per weight, to be added to the next step. A test (`test_optimizers.py`) takes steps
at a learning rate of one unit and checks them against torch's. This is the "wider fixed
point" that mattered: the weight stays Q12 for the forward pass, but behaves as if it were Q24.

**2. Adam had no bias correction (fixed, `b3e2f57`).** The moving averages start at zero, so
without the correction the first steps were about 3 times too large (`m / sqrt(v)` is 3.16 at
step 1) and weave's Adam did not take torch's: over three steps from the same weights the
weights differed from torch's by 80 percent of the distance they moved. With two numbers of state
per tensor (`1 - beta^n` in Q24) they agree within 5 percent, and the loss curve follows the
mirror's (above). It made no difference to the final accuracy on digits, and the old behaviour
was a faster start, so this is a fix for matching torch, not for accuracy.

**3. The softmax exponential was 20 percent low at -5 (fixed, `a279845`).** `(1 + x/64)^64` in Q12
gave exp(-1), exp(-2), exp(-5) and exp(-8) as 1485, 534, 22 and 0 in Q12 against 1507, 554, 28
and 1: 1.4, 3.7, 20 and 100 percent low (the documentation said 7 percent at -5). Now the
base is a cubic Taylor polynomial, in Q24, within 1 percent down to -16, and returns Q24. A
score gap above about 8.3 gives a probability of zero, because a Q12 probability under 1/4096 is
zero. In the trained digits network the right score is 7 to 12 above the runner-up (and the
rest further), so some classes of most samples had a probability of exactly zero: a margin of
about 8 in effect, losing under 1e-4 of probability per class. The exponential is now right
to -16, but the Q12 seed still rounds a probability under 1/4096 to zero unless the loss scale
(problem 4) is on.

**4. The softmax seed lost the small probabilities (fixed by the loss scale, `196fc75`).** The
seed of the backward pass is `p - t` with `p` a Q12 probability. When the loss is small, `1 - p` is
a few units and the wrong classes' probabilities are zero or one unit. weave's gradient was 4
percent smaller in norm than the float gradient at a training loss of 0.01 (relative error 6.7
percent), growing as the loss fell: 0.8 percent at a loss of 1, 1.4 at 0.24, 3.7 at 0.09, 6.7 at 0.011
(`gradient_error.py`). Rounding the probability instead of truncating it did not change this.
With a loss scale of 256 the error is 0.3 to 0.8 percent at every loss and the norm ratio is 1.00.

**5. Q12 adjoints underflow with small weights (fixed, `196fc75`).** Every layer on the way back
multiplies a Q12 adjoint by a weight and truncates, so with weights near 0.01 the gradient of a deep
network reaches the first layers as zero. With the initial weights of the depth 8 network
scaled by 0.01 and Adam at 0.0005, weave stayed at 16 percent test accuracy where the float mirror
reached 85 percent. The `loss_scale` option multiplies the seed by S and each optimizer divides
S out; with S = 16 weave reached 87 percent and with 256 85 percent. A unit test: four Adam steps on
two blocks of weights near 0.05, where weave's weights ended 77 percent of the distance
moved away from torch's without the scale and 2 percent away with it. The table of initial weight scales (accuracy after
30 epochs, one seed):

| optimizer | rate | init scale | float mirror | weave | weave x256 |
| --- | --- | --- | --- | --- | --- |
| Adam | 0.0005 | 0.01 | 0.851 | 0.163 | 0.849 |
| Adam | 0.0005 | 0.02 | 0.855 | 0.880 | 0.855 |
| Adam | 0.0005 | 0.05 | 0.902 | 0.840 | 0.906 |
| Adam | 0.0005 | 0.1 | 0.920 | 0.920 | 0.920 |
| sgd | 0.01 | 0.1 | 0.628 | 0.167 | 0.374 |
| sgd | 0.01 | 0.01 to 0.05 | 0.165 | 0.165 | 0.165 |

What the scale did not fix: sgd with weights at 0.1 still reaches 0.374 against 0.628. The float
network leaves its plateau and weave's scaled one leaves it more slowly. The default is 1
so that existing models compile as before; for the results above, 256 never hurt at normal
weights and Adam was 1.8 times slower with it.

At the other end the scale hurts: with weights 8 or 16 times the default, two epochs of tiny steps
end at a loss of 0.474 and 0.847 with a scale of 256 against 0.629 and 0.635 without and 0.543 and
0.676 for the float. I believe the scaled backward pass overflows there, but did not show it.

**6. RMSNorm returned zero for a large hidden layer (fixed, `e17faa4`).** The Newton chain for
`1 / sqrt(s)` converges for `s` up to 48, and above that the block was skipped and returned 0
(the documentation said it traps), so the layer output zeros, silently. Now `s` is shrunk by 64
or 4096 first, which reaches 196608; beyond that it still returns 0. In the table above, at
input scale 32 the test accuracy was 0.492 against the float's 0.684 and is now 0.670.

**7. Adam's integer square root searched 32 steps (fixed, `4fbff56`).** It searched the whole
range for every weight; now it starts from a bound on the size of its input. Bit-identical weights,
3.2 CPU seconds for 20 epochs of Adam down to 1.1.

**Open.**

- **Overflow does not trap, it wraps.** Arithmetic is 64-bit and wraps. A training run that
  diverges does not stop. In one run (silu, sgd 0.005) the gradients agreed with float to 0.2
  percent while they grew from 3 to 800000, then the float logits reached 1e33, past what a Q12 in 64 bits
  holds, and weave's gradients were garbage of about 1e12. The loss weave reports stays near 0.9
  from then on, since it is a squared probability error and bounded by 1, while the weights
  grew to 1e11 (a cross entropy of 1e170 once put back into torch). Nothing in the compiled program
  notices. A guard in the driver, failing when the loss of an epoch rises to many times its minimum,
  was not added.
- **Learning rates have a grid of 1/4096.** `rate * 4096` is rounded to a whole number, so 0.0005 is
  0.000488 and nothing below 0.000244 is possible. With the carry, small rates work, but they cannot
  be tuned finely. A Q16 rate would fix it and every generated call takes the rate.
- **Divergence at a high learning rate is as common as in float, and silent.** Ten seeds each, sgd on
  digits at depth 8: at rate 0.01 with tanh the float network diverged in 1 and weave in 0 (and in 2 of
  the older runs); at rate 0.005 with silu the float diverged in 1 and weave in 3 (2 with loss scale
  256). The runs are chaotic (a change in the last digit of the exponential changed which seeds
  diverged), so the honest reading is "the same order, weave perhaps a little more often", not a rate.
  The gradients that grow before such a spike agree with float to 0.2 percent until the values pass
  what a Q12 can hold, so the spikes are the dynamics, not the arithmetic.
- **Attention without softmax is unstable** in float and in weave; weave has no exponential inside a
  block.
- **The norm block is not proved reversible by Lean** (unchanged, noted in the reference), and
  `rsqrt` still returns 0 above 196608. The new exponential is proved by Lean in 7 seconds.
- **Not tried:** layers wider than 64, convolution or attention on the characters task, a GPU
  (the einsums can run on one), or a task where fixed point must matter, such as many more
  steps than 40 epochs.

## LayerNorm and invertible residual blocks, trained with Adam and keep_contraction

`bench/weave_residual_layernorm.py` (run from `crates/roop-weave/python` with `uv run --extra torch`,
`WEAVE_ROOP` and `ROOP_RT_LIB` set) trains, on scikit-learn's digits (offline, the split of the
digits section: 1344 train, 449 test), a network of a leapfrog layer followed by `blocks` pairs of
`Linear(64,32), LayerNorm(32, eps=1e-2), GELU, Linear(32,64)` and `x + F(x)` residual blocks
(`Linear(64,32), Tanh, Linear(32,64)`, weights scaled to a contraction). The comparison is the
float mirror, which is the function weave compiles (leapfrog state, the same fixed-point chain of
39 cells in the residual blocks) in doubles under torch's autograd and Adam, from the same initial
weights. Both take batches of 32 in order, summed loss, Adam 0.001 with `eps=1e-3`. Weave uses
`keep_contraction = 0.8`. A plain `nn.Sequential` forward is not this function (the state is a pair
and a residual block inverts a chain), so it is not used for the accuracy. One seed (0) only.

| model | epochs | float mirror test acc | weave test acc | weave train CE | weave cpu s | weave RSS MB | float cpu s | float RSS MB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 block | 20 | 0.9488 | 0.9488 (checked build) | 0.050 | 94.9 (checked) | 10 | 5.1 | 399 |
| 4 blocks | 30 | 0.9777 | 0.9777 | 0.0021 | 19.1 | 11 | 39.6 | 405 |

Per-epoch loss (half squared error of the probabilities per sample), 4 blocks: epochs 1, 2, 3, 10, 30
are 0.312, 0.083, 0.042, 0.016, 0.000 for the float mirror and 0.319, 0.089, 0.046, 0.013, 0.000 for
weave. Both have a bump around epochs 9 to 20 (loss up to 0.012) before settling. With one block,
weave's curve tracks the float curve to the third decimal for the first 3 epochs.

The `--checked` run (`[checks] overflow = true`, `roop weave --driver --checked`) of the 1 block
model finished 20 epochs without stopping, so no wrap in a step. The 4 block run was unchecked.

Honest caveats:

- One seed and one split; the test set is 449 images, so the equal accuracies prove nothing finer
  than a few images. The multi-seed run is `--seeds 0 1 2`, which I did not complete.
- Unexplained: after training, the spectral-norm bound of the residual weights (the Python bound
  of `contraction.py`, 1.77 for 4 blocks, 3.2 for 1 block) is above the 0.8 that `keep_contraction`
  asked for, while `project_contraction` on its own shrinks 64 by 32 weights correctly (checked in
  isolation), and the Python estimate of the bound matches numpy's spectral norm. The loss and
  accuracy are nonetheless those of the float mirror, whose own bound reaches 1.8 to 6.2. The
  bound is an upper bound on the Lipschitz constant, so a value above 1 does not make the chain
  diverge, but this means the cap is not what held the run, and I did not find why. Exporting
  the trained model again is refused by `torch_to_weave` for the same reason, so the script
  evaluates through a mirror built with the limit lifted.
- Build time dominates: `roop build` of the generated training program took 90 to 140 s for these
  models, against 2 to 20 s of training, so there is no cheap regression test for this model.
- Memory is the compiled program's peak resident size against the whole torch process (about
  400 MB, mostly torch itself), not activation memory.

## What this says

On these tasks fixed point is not what limits accuracy. After the seven fixes weave reaches the
accuracy of float training, with no stored activations, at a memory of about the parameters, and
a speed no worse than an ordinary torch MLP of the same size. Before them it was worse in the
regimes that real tasks visit and XOR does not: small learning rates, Adam's first steps, small
weights and large inputs. All of those were precision in the update or on the way back, not the
Q12 forward pass, whose logits differ from float by about 0.01 and whose accuracy is identical on the
test set. What remains is the usual fragility of unnormalized deep nets at a high rate, which weave
shares, and the silence of its failures.

## Commits

On `feat/weave-real-models`: `b2cfdaf` the experiments, `b3e2f57` Adam, `a279845` the
exponential, `e03bfd7` the carry, `8684362` the digits test and the output scale, `e17faa4` rsqrt,
`8b9075e` the fixed point evaluation, `4fbff56` the square root, `8b75821` the gradient
measurement, `196fc75` the loss scale and the Q24 exponential. The results directory and this
document are the last commit.
