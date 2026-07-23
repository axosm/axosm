pub mod galaxy;
pub mod seed;
pub mod star_system;
pub mod universe;

// The Domain-Driven Approach
// We keep the logic inside files named after the scale they represent (universe.rs, galaxy.rs, star_system.rs, planet.rs).

// universe.rs: Manages the macro-scale. It queries cosmic density to decide where galaxies spawn.

// galaxy.rs: Contains struct Galaxy. It takes a coordinate, checks if it should exist, and calculates galaxy-specific density (like spiral arms) to determine where star systems spawn.

// star_system.rs: Contains struct StarSystem. It determines how many planets spawn based on local system density/mass.

// planet.rs: Contains struct Planet. Handles biomes and terrain generation.

// It keeps your code perfectly encapsulated. universe.rs doesn't need to know how a planet is generated; it only cares about galaxies.
