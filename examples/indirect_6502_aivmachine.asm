STDIN = $2000
STDOUT = $2001
EXIT = $2003

.macro set_src address
	LDA #<address
	STA src_low
	LDA #>address
	STA src_high
.endmacro

.macro set_dst address
	LDA #<address
	STA dst_low
	LDA #>address
	STA dst_high
.endmacro

.macro set address, value
	LDA #value
	STA address
.endmacro

.macro set16 address, value
	LDA #<value
	STA <address
	LDA #>value
	STA >address
.endmacro

.segment "CODE"

set seconds, 1
JSR sleep

main:
set_src hello
JSR printstring

set_dst yourname
JSR readline

set_src hi
JSR printstring

set_src space
JSR printstring

set_src yourname
JSR printstring_nl

set seconds, 2
JSR sleep

JMP main

LDA #0
STA EXIT

hello:
.byte "Hello, what is your name? ",0
wave:
.byte "Wave",0
yep:
.byte "Yep",0
nope:
.byte "Nope",0
hi:
.byte "Hi",0
space:
.byte " ",0

.macro enter
	PHA
        TXA
        PHA
        TYA
        PHA
.endmacro

.macro exit
	PLA
        TAY
        PLA
        TAX
        PLA
        RTS	
.endmacro

.macro copy16 source, destination
	LDA source
	STA destination
	LDA source+1
	STA destination+1
.endmacro


printstring:
	enter
	LDY #$00
printstring_loop:
	LDA (<src),Y
	BEQ printstring_end
	STA STDOUT
	INY
	JMP printstring_loop	
printstring_end:
	exit

printstring_nl:
	JSR printstring
	PHA
	LDA #$0a
	STA STDOUT
	PLA
	RTS

readline:
	enter
	LDY #0
readline_loop:
	LDA STDIN	
	BEQ readline_loop
	CMP #$0A
	BEQ readline_end
	STA (<dst),Y
	INY
	JMP readline_loop
readline_end:
	LDA #0
	STA (<dst),Y
	exit
	
add16:
	enter
	CLC
	LDA arg0_low
	ADC arg1_low
	STA res0_low
	LDA arg0_high
	ADC arg1_high
	STA res0_high
	exit
sub16:
	enter
	SEC
	LDA arg0_low
	SBC arg1_low
	STA res0_low
	LDA arg0_high
	SBC arg1_high
	STA res0_high
	exit

div16:
	enter
	set16 arg0, 17000
	set16 arg1, 1000
div16_loop:
	JSR sub16
	copy16 res0, arg0
	JMP div16_loop


print16:
	enter
	

sleep:
	enter
	LDA counter
	LDY seconds
	BEQ sleep_end; check for 0 value
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
	BEQ sleep_end
	LDA counter
	LDX #60
	jmp sleep_loop
sleep_end:
	exit
	
vblank:
	INC counter
	RTI

.segment "ZP"
src:
src_low:
.byte 0
src_high:
.byte 0
dst:
dst_low:
.byte 0
dst_high:
.byte 0
arg0:
arg0_low:
.byte 0
arg0_high:
.byte 0
arg1:
arg1_low:
.byte 0
arg1_high:
.byte 0
res0:
res0_low:
.byte 0
res0_high:
.byte 0
counter:
.byte 0
seconds:
.byte 0

.segment "RAM"
.res 512,0
yourname:
.res 100,0

.segment "VECTORS"
.word vblank
