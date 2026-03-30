with open("src/layer1/drone_tests.rs", "r") as f:
    if "test_drones_deactivate_without_power_or_bandwidth" in f.read():
        print("Test is present")
    else:
        print("Test is missing")
