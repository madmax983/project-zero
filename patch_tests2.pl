#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/infrastructure/mod.rs';
open my $fh, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$fh> };
close $fh;

$content =~ s/mod tests \{/mod tests {\n    use super::*;\n    use crate::layer1::*; /;

open my $out, '>', $file or die "Cannot write $file: $!";
print $out $content;
close $out;
