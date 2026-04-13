import sys

def main():
    with open('src/layer1/systems/observation.rs', 'r') as f:
        content = f.read()

    target = "crate::layer1::integration::mass_driver_chronicle_bridge\n                .after(crate::layer1::logistics::mass_driver::package_arrival_system),"
    replacement = target + """
            crate::layer1::integration::predatory_weather_emission_bridge_system,
            crate::layer1::integration::predatory_weather_impact_bridge_system,"""

    if "predatory_weather_emission_bridge_system" not in content:
        content = content.replace(target, replacement)
        with open('src/layer1/systems/observation.rs', 'w') as f:
            f.write(content)
        print("Patched observation.rs")
    else:
        print("Already patched")

if __name__ == "__main__":
    main()
