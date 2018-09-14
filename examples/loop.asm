.ORG $8000

; check overflow
LDX #$00;
DEX

CLI ; enable IRQ

; activate timer
LDA #$ef
STA $d000

loop:
	JMP loop
