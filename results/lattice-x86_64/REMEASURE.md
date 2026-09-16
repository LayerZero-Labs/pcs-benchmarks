# Lattice remeasurement status

## Akita upstream-pin refresh pending

The harness now pins Akita `c0cb822f28b7b9efe85b1924b029d36e13cdf516`
and loads its upstream schedule artifacts without vendor patches. Existing
Akita records still identify the previous implementation and remain historical;
they must be replaced by the new run before regenerating combined reports with
this harness. The raw records have not been relabeled.
