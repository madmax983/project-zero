#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/infrastructure/mod.rs';
open my $fh, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$fh> };
close $fh;

# Replace individual tests with separate modules
$content =~ s/mod tests \{/#[cfg(test)]\nmod building_gate_test {\n    use super::*;\n    use crate::layer1::*; \n    include!("building_gate_test.rs");\n}\n\n#[cfg(test)]\nmod structure_fragile_tests {\n    use super::*;\n    use crate::layer1::*; \n    include!("structure_fragile_tests.rs");\n}\n\n#[cfg(test)]\nmod structure_jury_rig_tests {\n    use super::*;\n    use crate::layer1::*; \n    include!("structure_jury_rig_tests.rs");\n}\n\n#[cfg(test)]\nmod structure_maintenance_tests {\n    use super::*;\n    use crate::layer1::*; \n    include!("structure_maintenance_tests.rs");\n}\n\n#[cfg(test)]\nmod work_building_tests {\n    use super::*;\n    use crate::layer1::*; \n    include!("work_building_tests.rs");\n}\n\n#[cfg(test)]\nmod shift_integration_tests {\n    use super::*;\n    use crate::layer1::*; \n    include!("shift_integration_tests.rs");\n}\n\n\/\*/;

open my $out, '>', $file or die "Cannot write $file: $!";
print $out $content;
close $out;
