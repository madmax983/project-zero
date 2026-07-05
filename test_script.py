import re

def main():
    with open('src/layer1/tech/rhythm.rs', 'r') as f:
        content = f.read()

    print(content)

if __name__ == '__main__':
    main()
