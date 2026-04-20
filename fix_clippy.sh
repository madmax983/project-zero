sed -i 's/if resources.try_deduct(&cost) {/pet.is_starving = !resources.try_deduct(\&cost);/' src/layer1/social/xenoflora_pet.rs
sed -i '/pet.is_starving = false;/d' src/layer1/social/xenoflora_pet.rs
sed -i '/} else {/d' src/layer1/social/xenoflora_pet.rs
sed -i '/pet.is_starving = true;/d' src/layer1/social/xenoflora_pet.rs
# Wait, sed -i might delete too many lines. Let's do it safely.
