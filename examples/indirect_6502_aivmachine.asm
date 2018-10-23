STDOUT = $2001
EXIT = $2003

.segment "CODE"

LDA #<hello
STA src_low
LDA #>hello
STA src_high
JSR printstring

LDA #1
STA seconds
JSR sleep

LDA #<yep
STA src_low
LDA #>yep
STA src_high
JSR printstring

LDA #2
STA seconds
JSR sleep

LDA #<nope
STA src_low
LDA #>nope
STA src_high
JSR printstring_nl

LDA #0
STA EXIT

hello:
.byte "Hello",10,0
wave:
.byte "Wave",10,0
yep:
.byte "Yep",10,0
nope:
.byte "Nope",0

printstring:
	PHA
	TXA
	PHA
	TYA
	PHA
	LDY #$00
printstring_loop:
	LDA (<src),Y
	BEQ return
	STA STDOUT
	INY
	JMP printstring_loop	

printstring_nl:
	JSR printstring
	PHA
	LDA #$0a
	STA STDOUT
	PLA
	RTS

sleep:
	PHA
        TXA
        PHA
        TYA
        PHA
	LDA counter
	LDY seconds
	BEQ return; check for 0 value
	LDX #60
sleep_loop:
	CMP counter
	BEQ sleep_loop
	DEX
	BEQ sleep_next
	LDA counter
	JMP sleep_loop
sleep_next:
	DEY
	BEQ return
	LDA counter
	LDX #60
	jmp sleep_loop
	
return:
	PLA
        TAY
        PLA
        TAX
        PLA
        RTS	

vblank:
	INC counter
	RTI
	

.segment "ZP"
src:
src_low:
.byte 0
src_high:
.byte 0
dst_low:
.byte 0
dst_high:
.byte 0
counter:
.byte 0
seconds:
.byte 0

.segment "VECTORS"
.word vblank
