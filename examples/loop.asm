.ORG $C000

; check overflow
LDX #$00;
DEX

CLI ; enable IRQ

; activate timer
LDA #$ef
STA $b000

loop:
	JMP loop

nmi:
	RTI

reset:
	RTI

irq:
	RTI

.SEGMENT "VECTORS"

.WORD nmi ; $fffa
.WORD reset ; $fffc
.WORD irq ; $fffe
