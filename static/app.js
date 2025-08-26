class CalendarMonitor {
    constructor() {
        this.ws = null;
        this.reconnectInterval = null;
        this.reconnectDelay = 1000;
        this.maxReconnectDelay = 30000;
        
        this.init();
    }

    init() {
        this.updateCurrentTime();
        this.connectWebSocket();
        
        // Update current time every second
        setInterval(() => this.updateCurrentTime(), 1000);
    }

    updateCurrentTime() {
        const now = new Date();
        const timeString = now.toLocaleTimeString('en-US', {
            hour12: false,
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
        const dateString = now.toLocaleDateString('en-US', {
            weekday: 'long',
            year: 'numeric',
            month: 'long',
            day: 'numeric'
        });
        
        document.getElementById('currentTime').textContent = `${dateString} - ${timeString}`;
    }

    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        try {
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                console.log('WebSocket connected');
                this.updateConnectionStatus(true);
                this.reconnectDelay = 1000; // Reset reconnect delay
            };
            
            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.updateMeetingDisplay(data);
                } catch (error) {
                    console.error('Error parsing WebSocket message:', error);
                }
            };
            
            this.ws.onclose = () => {
                console.log('WebSocket disconnected');
                this.updateConnectionStatus(false);
                this.scheduleReconnect();
            };
            
            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.updateConnectionStatus(false);
            };
            
        } catch (error) {
            console.error('Error creating WebSocket:', error);
            this.updateConnectionStatus(false);
            this.scheduleReconnect();
        }
    }

    scheduleReconnect() {
        if (this.reconnectInterval) {
            clearTimeout(this.reconnectInterval);
        }
        
        this.reconnectInterval = setTimeout(() => {
            console.log('Attempting to reconnect...');
            this.connectWebSocket();
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
        }, this.reconnectDelay);
    }

    updateConnectionStatus(connected) {
        const statusIndicator = document.getElementById('connectionStatus');
        const connectionText = document.getElementById('connectionText');
        
        if (connected) {
            statusIndicator.textContent = '🟢';
            statusIndicator.classList.add('connected');
            connectionText.textContent = 'Connected';
        } else {
            statusIndicator.textContent = '🔴';
            statusIndicator.classList.remove('connected');
            connectionText.textContent = 'Disconnected - Reconnecting...';
        }
    }

    updateMeetingDisplay(data) {
        this.updateCurrentMeeting(data.current_meeting, data.countdown_seconds);
        this.updateNextMeeting(data.next_meeting);
        this.updateActiveTimeBlocks(data.active_time_blocks);
    }

    updateCurrentMeeting(meeting, countdownSeconds) {
        const noMeetingDiv = document.getElementById('noCurrentMeeting');
        const meetingInfoDiv = document.getElementById('currentMeetingInfo');
        const meetingCard = document.getElementById('currentMeetingCard');
        
        if (!meeting) {
            noMeetingDiv.style.display = 'block';
            meetingInfoDiv.style.display = 'none';
            meetingCard.classList.remove('urgent');
            return;
        }
        
        noMeetingDiv.style.display = 'none';
        meetingInfoDiv.style.display = 'block';
        
        // Update meeting details
        document.getElementById('currentMeetingTitle').textContent = meeting.title;
        document.getElementById('currentMeetingTime').textContent = this.formatTimeRange(meeting.start_time, meeting.end_time);
        document.getElementById('currentMeetingLocation').textContent = meeting.location || 'No location specified';
        document.getElementById('currentMeetingDescription').textContent = meeting.description || '';
        
        // Update countdown
        const countdownElement = document.getElementById('currentMeetingCountdown');
        if (countdownSeconds && countdownSeconds > 0) {
            countdownElement.textContent = this.formatCountdown(countdownSeconds);
            
            // Add urgent styling if less than 5 minutes remaining
            if (countdownSeconds <= 300) {
                meetingCard.classList.add('urgent');
            } else {
                meetingCard.classList.remove('urgent');
            }
        } else {
            countdownElement.textContent = '00:00';
            meetingCard.classList.remove('urgent');
        }
        
        // Add update animation
        meetingCard.classList.add('updating');
        setTimeout(() => meetingCard.classList.remove('updating'), 300);
    }

    updateNextMeeting(meeting) {
        const noMeetingDiv = document.getElementById('noNextMeeting');
        const meetingInfoDiv = document.getElementById('nextMeetingInfo');
        const meetingCard = document.getElementById('nextMeetingCard');
        
        if (!meeting) {
            noMeetingDiv.style.display = 'block';
            meetingInfoDiv.style.display = 'none';
            return;
        }
        
        noMeetingDiv.style.display = 'none';
        meetingInfoDiv.style.display = 'block';
        
        // Update meeting details
        document.getElementById('nextMeetingTitle').textContent = meeting.title;
        document.getElementById('nextMeetingTime').textContent = this.formatTimeRange(meeting.start_time, meeting.end_time);
        document.getElementById('nextMeetingLocation').textContent = meeting.location || 'No location specified';
        document.getElementById('nextMeetingDescription').textContent = meeting.description || '';
        
        // Check if meeting is not today and show date if needed
        const meetingDate = new Date(meeting.start_time);
        const today = new Date();
        const isToday = meetingDate.toDateString() === today.toDateString();
        
        const dateElement = document.getElementById('nextMeetingDate');
        if (!isToday) {
            // Show date for meetings not today
            const dateStr = this.formatMeetingDate(meetingDate, today);
            dateElement.textContent = dateStr;
            dateElement.style.display = 'block';
        } else {
            // Hide date for today's meetings
            dateElement.style.display = 'none';
        }
        
        // Calculate and display duration
        const duration = this.calculateDuration(meeting.start_time, meeting.end_time);
        document.getElementById('nextMeetingDuration').textContent = `Duration: ${duration}`;
        
        // Calculate time until meeting starts
        const timeUntilStart = this.calculateTimeUntilStart(meeting.start_time);
        document.getElementById('nextMeetingTimeUntil').textContent = timeUntilStart;
    }

    updateActiveTimeBlocks(timeBlocks) {
        const noTimeBlockDiv = document.getElementById('noActiveTimeBlocks');
        const timeBlockInfoDiv = document.getElementById('activeTimeBlockInfo');
        const timeBlockBar = document.getElementById('timeBlockBar');
        
        if (!timeBlocks || timeBlocks.length === 0) {
            noTimeBlockDiv.style.display = 'flex';
            timeBlockInfoDiv.style.display = 'none';
            return;
        }
        
        // Show the first active time block
        const timeBlock = timeBlocks[0];
        
        noTimeBlockDiv.style.display = 'none';
        timeBlockInfoDiv.style.display = 'flex';
        
        // Extract time block name (remove brackets)
        const timeBlockName = timeBlock.title.startsWith('[') && timeBlock.title.endsWith(']') 
            ? timeBlock.title.slice(1, -1) 
            : timeBlock.title;
        
        // Update time block information
        document.getElementById('activeTimeBlockTitle').textContent = timeBlockName;
        document.getElementById('activeTimeBlockTime').textContent = this.formatTimeRange(timeBlock.start_time, timeBlock.end_time);
        
        // Calculate and display countdown
        const endTime = new Date(timeBlock.end_time);
        const now = new Date();
        const timeLeft = Math.max(0, Math.floor((endTime - now) / 1000));
        const countdownText = this.formatCountdown(timeLeft);
        document.getElementById('activeTimeBlockCountdown').textContent = countdownText;
    }

    formatTimeRange(startTime, endTime) {
        const start = new Date(startTime);
        const end = new Date(endTime);
        
        const startStr = start.toLocaleTimeString('en-US', {
            hour12: false,
            hour: '2-digit',
            minute: '2-digit'
        });
        
        const endStr = end.toLocaleTimeString('en-US', {
            hour12: false,
            hour: '2-digit',
            minute: '2-digit'
        });
        
        return `${startStr} - ${endStr}`;
    }

    formatCountdown(seconds) {
        if (seconds <= 0) return '00:00';
        
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const remainingSeconds = seconds % 60;
        
        if (hours > 0) {
            return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
        } else {
            return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
        }
    }

    calculateDuration(startTime, endTime) {
        const start = new Date(startTime);
        const end = new Date(endTime);
        const durationMs = end - start;
        const minutes = Math.floor(durationMs / (1000 * 60));
        
        if (minutes >= 60) {
            const hours = Math.floor(minutes / 60);
            const remainingMinutes = minutes % 60;
            return `${hours}h ${remainingMinutes}m`;
        } else {
            return `${minutes}m`;
        }
    }

    formatMeetingDate(meetingDate, today) {
        const tomorrow = new Date(today);
        tomorrow.setDate(today.getDate() + 1);
        
        // Check if it's tomorrow
        if (meetingDate.toDateString() === tomorrow.toDateString()) {
            return 'Tomorrow';
        }
        
        // Check if it's this week (within 7 days)
        const daysDiff = Math.floor((meetingDate - today) / (1000 * 60 * 60 * 24));
        
        if (daysDiff >= 2 && daysDiff <= 7) {
            // Show day name for this week
            return meetingDate.toLocaleDateString('en-US', { 
                weekday: 'long',
                month: 'short',
                day: 'numeric'
            });
        }
        
        // Show full date for further dates
        return meetingDate.toLocaleDateString('en-US', {
            weekday: 'short',
            month: 'short',
            day: 'numeric',
            year: 'numeric'
        });
    }

    calculateTimeUntilStart(startTime) {
        const start = new Date(startTime);
        const now = new Date();
        const timeDiff = start - now;
        
        if (timeDiff <= 0) return 'Starting now';
        
        const seconds = Math.floor(timeDiff / 1000);
        return this.formatCountdown(seconds);
    }
}

// Initialize the application when the page loads
document.addEventListener('DOMContentLoaded', () => {
    new CalendarMonitor();
});
