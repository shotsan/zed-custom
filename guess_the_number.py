#!/usr/bin/env python3
import random

def print_banner():
    banner = """
    ╔═══════════════════════════════════╗
    ║   🎮 GUESS THE NUMBER GAME 🎮    ║
    ╚═══════════════════════════════════╝
    """
    print(banner)

def print_celebration():
    celebration = """
    
    ★ ･ﾟ✧ *:･ﾟ✧ *:･ﾟ✧ *:･ﾟ✧
         YOU WIN! 🎉
    ★ ･ﾟ✧ *:･ﾟ✧ *:･ﾟ✧ *:･ﾟ✧
    """
    print(celebration)

def main():
    print_banner()
    print("I'm thinking of a number between 1 and 100...")
    print("Can you guess it?\n")
    
    secret_number = random.randint(1, 100)
    attempts = 0
    max_attempts = 7
    
    while attempts < max_attempts:
        remaining = max_attempts - attempts
        print(f"💡 Attempts remaining: {remaining}")
        
        try:
            guess = int(input("Enter your guess: "))
        except ValueError:
            print("❌ Please enter a valid number!\n")
            continue
        
        attempts += 1
        
        if guess < 1 or guess > 100:
            print("🚫 Please guess between 1 and 100!\n")
            attempts -= 1
            continue
        
        if guess == secret_number:
            print_celebration()
            print(f"You got it in {attempts} attempts!")
            return
        elif guess < secret_number:
            diff = secret_number - guess
            if diff <= 5:
                print("🔥 SO CLOSE! Go higher!\n")
            else:
                print("📈 Too low! Go higher!\n")
        else:
            diff = guess - secret_number
            if diff <= 5:
                print("🔥 SO CLOSE! Go lower!\n")
            else:
                print("📉 Too high! Go lower!\n")
    
    print(f"\n💀 Game Over! The number was {secret_number}")
    print("Better luck next time!")

if __name__ == "__main__":
    main()
