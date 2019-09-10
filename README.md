# Graph-Builder
An algorithm for generating graphs with post-filtering and edge composition.

This algorithm is used by automated theorem provers on typical group/category problems.
For example, to study algebra or path semantics.

The advantage of this algorithm is that it exploits symmetry of Category Theory.
For every morphism `A -> B` and `B -> C`, there exists a morphism `A -> C`.
This means that for a large number of objects, one only needs to keep neighbour morphisms.

Constructing a graph using small operations makes it possible to
minimize the work required to get from one node to another.

For information of how use this library, see the documentation on the various functions.
