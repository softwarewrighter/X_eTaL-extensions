// Full pinned Nano 1K calculator, external contacts, production counters.
// Nominal 27 MHz oscillator (1 ps simulation precision); no UART stimulus.
`timescale 1ns/1ps
module tb;
  reg clk=0;
  always #18.5185 clk=~clk;
  reg [15:0] pressed=0;
  reg [3:0] ops_n=15;
  reg [3:0] cols_n=15;
  tri1 [3:0] rows;
  wire din,sclk,load,oe;
  wire [2:0] leds;
  hex1000_calculator dut(clk,cols_n,ops_n,rows,din,sclk,load,oe,leds);
  integer row;
  always @* begin
    cols_n=15;
    for(row=0;row<4;row=row+1)
      if(rows[row]===1'b0) cols_n=cols_n & ~pressed[row*4 +:4];
  end
  wire reset=dut.rst;
  wire machine_clk=dut.machine_clk;
  wire [3:0] a=dut.machine.n1[3:0], y=dut.machine.n2[3:0];
  wire [1:0] x=dut.machine.n3[1:0];
  wire status=dut.machine.n4;
  wire [5:0] pc=dut.machine.n5[5:0], sr=dut.machine.n8[5:0];
  wire [3:0] pa=dut.machine.n6[3:0], pb=dut.machine.n7[3:0];
  wire call_latch=dut.machine.n9;
  wire [5:0] o=dut.o;
  wire [15:0] r=dut.r;
  wire [4:0] k=dut.k;
  wire [39:0] shown=dut.panel.display_digits;
  wire [3:0] scan_rows=dut.rows;
  wire [3:0] cols_sync=dut.cols_sync, ops_sync=dut.ops_sync;
  wire [19:0] accepted=dut.panel.accepted;
  wire [2:0] stable=dut.panel.stable;
  wire [1:0] phase=dut.panel.phase;
  wire [4:0] key_code=dut.panel.key;
  wire have_key=dut.panel.have_key;
  integer stage=0, accepted_count=0, cycles=0;
  reg [3:0] old_cols,old_ops;
  reg [4:0] old_k;
  integer trace,events,index;
  wire [63:0] field[0:37];
  assign field[0]=old_cols; assign field[1]=old_ops; assign field[2]=old_k;
  assign field[3]=a; assign field[4]=y; assign field[5]=x; assign field[6]=status;
  assign field[7]=pc; assign field[8]=pa; assign field[9]=pb; assign field[10]=sr;
  assign field[11]=call_latch; assign field[12]=o; assign field[13]=r;
  assign field[14]=dut.panel.tick; assign field[15]=dut.panel.row;
  assign field[16]=dut.panel.scan; assign field[17]=dut.panel.candidate;
  assign field[18]=stable; assign field[19]=accepted;
  assign field[20]=phase; assign field[21]=key_code; assign field[22]=dut.panel.last_equals;
  assign field[23]=dut.panel.digits;
  assign field[24]=dut.panel.spi_phase; assign field[25]=dut.panel.word_index;
  assign field[26]=dut.panel.shift; assign field[27]=din; assign field[28]=sclk; assign field[29]=load;
  assign field[30]=have_key; assign field[31]=k;
  assign field[33]=pressed; assign field[34]=ops_n;
  assign field[35]=cols_sync; assign field[36]=ops_sync; assign field[37]=stage;
  reg [63:0] clock_ns;
  assign field[32]=clock_ns;
  always @(posedge machine_clk) begin
    if(!reset) begin
      old_cols=cols_sync; old_ops=ops_sync; old_k=k;
      if(phase==0 && accepted!=0) accepted_count=accepted_count+1;
      #1;
      clock_ns=$time;
      for(index=0;index<38;index=index+1) $fwrite(trace,"%0d ",field[index]);
      $fwrite(trace,"\n"); cycles=cycles+1;
    end
  end
  task clocks; input integer n; integer i;
    begin for(i=0;i<n;i=i+1) @(negedge machine_clk); end
  endtask
  task ready; integer tries;
    begin tries=0; while(!(r[15] && phase==0) && tries<100000) begin clocks(1);tries=tries+1;end
      if(tries==100000) $fatal(1,"ready timeout");
    end
  endtask
  task key; input integer code; integer count_before,tries;
    begin ready; count_before=accepted_count;
      $fdisplay(events,"%0d %0d %0d",$time,stage,code);
      if(code<16) pressed=16'b1<<((3-code/4)*4+code%4);
      else ops_n=~(4'b1<<(code-16));
      tries=0; while(phase!=3 && tries<100000) begin clocks(1);tries=tries+1;end
      if(tries==100000) $fatal(1,"key acceptance timeout");
      clocks(1024);
      if(accepted_count!=count_before+1) $fatal(1,"held key repeated");
      pressed=0;ops_n=15;ready;
    end
  endtask
  reg [15:0] word=0;
  integer bits=0,words=0;
  reg [7:0] display[0:7];
  always @(posedge sclk) begin
    if(load!==0) $fatal(1,"SPI clock with LOAD high");
    word={word[14:0],din};bits=bits+1;
  end
  always @(posedge load) if(bits!=0) begin
    if(bits!=16) $fatal(1,"SPI word length");
    if(word[15:8]>=1 && word[15:8]<=8) display[word[15:8]-1]=word[7:0];
    else if(word!=16'h0c01 && word!=16'h0f00 && word!=16'h0900 && word!=16'h0b07 && word!=16'h0a04)
      $fatal(1,"invalid initialization");
    bits=0;words=words+1;
  end
  task screen; input [63:0] want;
    begin clocks(1000);
      if({display[7],display[6],display[5],display[4],display[3],display[2],display[1],display[0]}!==want)
        $fatal(1,"decoded MAX7219 screen differs");
    end
  endtask
  integer codes[0:9]; integer i;
  initial begin
    trace=$fopen("trace.txt","w");events=$fopen("events.txt","w");
    $dumpfile("run.vcd");
    $dumpvars(0,reset,machine_clk,pressed,ops_n,cols_n,scan_rows,cols_sync,ops_sync,
                 accepted,stable,phase,key_code,have_key,shown,din,sclk,load,oe,a,y,x,status,pc,pa,pb,sr,call_latch,o,r,k,stage);
    wait(!reset); ready; screen(64'h3d4f773d1f4f4f47);
    stage=1;pressed=16'h1000;clocks(32);pressed=0;clocks(4096);
    if(accepted_count!=0 || have_key) $fatal(1,"bounce accepted or banner cleared");
    screen(64'h3d4f773d1f4f4f47);
    $display("PASS startup banner and rejected bounce");
    codes[0]=12;codes[1]=0;codes[2]=13;codes[3]=14;codes[4]=16;
    codes[5]=11;codes[6]=14;codes[7]=14;codes[8]=15;codes[9]=19;
    for(i=0;i<10;i=i+1) begin stage=10+i; key(codes[i]); end
    stage=25;screen(64'h0000003070474e3d);
    if(dut.panel.digits!=={15'b0,5'd17,5'd23,5'd31,5'd28,5'd29}) $fatal(1,"17FCD digit latches");
    $display("PASS C0DE+BEEF=17FCD from physical contacts and MAX7219 bytes");
    stage=30;key(19);stage=31;screen(64'h000000000000007e);
    if(accepted_count!=11 || !oe) $fatal(1,"acceptance count or OE");
    $display("PASS == clears, held-key suppression, %0d accepted keys, %0d SPI words, %0d clocks",accepted_count,words,cycles);
    $fclose(trace);$fclose(events);$finish;
  end
  initial begin #1000000000; $fatal(1,"simulation watchdog"); end
endmodule
