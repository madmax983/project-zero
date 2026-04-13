import sys

def main():
    with open('tests/integration/predatory_weather.rs', 'r') as f:
        content = f.read()

    content = content.replace(
        "use scale::layer1::core::integration::{predatory_weather_emission_bridge_system, predatory_weather_impact_bridge_system};",
        "use scale::layer1::integration::{predatory_weather_emission_bridge_system, predatory_weather_impact_bridge_system};"
    )

    with open('tests/integration/predatory_weather.rs', 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()
