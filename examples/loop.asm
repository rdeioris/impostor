.ORG $8000

; check overflow
LDX #$00;
DEX

; activate timer
LDA #$ef
STA $d000

loop:
	JMP loop
